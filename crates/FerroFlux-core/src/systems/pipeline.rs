use crate::components::execution_state::{ActiveWorkflowState, DataRef};
use crate::components::pipeline::PipelineNode;
use crate::resources::registry::DefinitionRegistry;
use crate::tools::ToolContext;
use crate::tools::registry::ToolRegistry;
use anyhow::Result;
use bevy_ecs::prelude::*;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use jmespath;
use serde_json::Value;
use std::collections::HashMap;

/// System that executes PipelineNodes when triggered.
///
/// NOTE: In a real implementation, this would likely be an async system or spawned task.
/// For this MVP, we execute synchronously when an "Exec" signal is received (implied).
#[allow(clippy::type_complexity)]
pub fn pipeline_execution_system(
    mut query: Query<(
        Entity,
        &mut PipelineNode,
        &mut crate::components::Inbox,
        &mut crate::components::Outbox,
        Option<&crate::components::shadow::ShadowExecution>,
        Option<&crate::components::NodeConfig>,
    )>,
    node_registry: Res<DefinitionRegistry>,
    tool_registry: Res<ToolRegistry>,
    store: Res<crate::store::BlobStore>,
    bus: Res<crate::api::events::SystemEventBus>,
    secret_store: Res<crate::secrets::DatabaseSecretStore>,
    runtime: Res<crate::resources::TokioRuntime>,
) {
    const MAX_ITEMS_PER_TICK: usize = 50;
    for (_entity, mut node, mut inbox, mut outbox, shadow_exec, node_config) in query.iter_mut() {
        let mut processed_count = 0;
        while processed_count < MAX_ITEMS_PER_TICK {
            let ticket = match inbox.queue.pop_front() {
                Some(t) => t,
                None => break,
            };
            processed_count += 1;
            // 1. Load Data/Context
            if let Ok(data) = store.claim(&ticket) {
                let mut state: ActiveWorkflowState = if let Ok(s) = serde_json::from_slice(&data) {
                    s
                } else {
                    // Fallback: Assume raw input payload (e.g. from Webhook)
                    let mut s = ActiveWorkflowState::new();
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&data) {
                        s.merge(val);
                    }
                    s
                };

                // 2. Execute
                let mut memory = HashMap::new(); // Global memory stub
                // 3. Output
                let active_ports = match execute_pipeline_node(
                    &mut node,
                    &mut state,
                    &node_registry,
                    &tool_registry,
                    &mut memory,
                    ticket.metadata.get("trace_id").cloned().unwrap_or_default(),
                    Some(bus.clone()),
                    Some(&store),
                    shadow_exec,
                    node_config,
                    Some(&secret_store),
                    Some(&runtime),
                ) {
                    Ok(ports) => ports,
                    Err(e) => {
                        let t_id = ticket.metadata.get("trace_id").cloned().unwrap_or_default();
                        tracing::error!(trace_id = %t_id, "Pipeline execution failed: {}", e);

                        // Emit NodeError if we have a bus and trace_id
                        if !t_id.is_empty() {
                            let _ = bus.0.send(crate::api::events::SystemEvent::NodeError {
                                trace_id: t_id,
                                node_id: node_config.map(|c| c.id).unwrap_or_default(),
                                error: e.to_string(),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            });
                        }

                        // Implement Formalized Error Handling:
                        // 1. Inject error into state
                        state.set(
                            "_error",
                            serde_json::json!({
                                "message": e.to_string(),
                                "node_id": node_config.map(|c| c.id).unwrap_or_default()
                            }),
                        );

                        // 2. Return standard error port to allow flow recovery
                        vec!["_error".to_string()]
                    }
                };

                // OPTIMIZATION: "The Spill Strategy"
                state.optimize_memory(&store, 1024);

                // Serialize State
                if let Ok(new_bytes) = serde_json::to_vec(&state) {
                    for port in active_ports {
                        // Check in for each port? Or reuse?
                        // Reuse same data ticket is fine if immutable.
                        // But we might want unique tickets for tracing paths?
                        // Reusing ticket is more efficient.
                        // Store.check_in dedupes by hash anyway.
                        if let Ok(new_ticket) =
                            store.check_in_with_metadata(&new_bytes, ticket.metadata.clone())
                        {
                            outbox.queue.push_back((Some(port), new_ticket));
                        }
                    }
                } else {
                    tracing::error!(
                        "Failed to serialize workflow state for ticket {:?}",
                        ticket.id
                    );
                }
            } else {
                tracing::error!("Failed to claim ticket from BlobStore: {:?}", ticket.id);
            }
        }
    }
}

/// Helper function to execute a single pipeline node.
/// This would be called by the main graph traversal loop.
#[allow(clippy::too_many_arguments)]
pub fn execute_pipeline_node(
    node: &mut PipelineNode,
    workflow_state: &mut ActiveWorkflowState,
    definitions: &DefinitionRegistry,
    tools: &ToolRegistry,
    global_memory: &mut HashMap<String, Value>,
    trace_id: String,
    event_bus: Option<crate::api::events::SystemEventBus>,
    store: Option<&crate::store::BlobStore>,
    shadow_exec: Option<&crate::components::shadow::ShadowExecution>,
    node_config: Option<&crate::components::NodeConfig>,
    secret_store: Option<&crate::secrets::DatabaseSecretStore>,
    runtime: Option<&crate::resources::TokioRuntime>,
) -> Result<Vec<String>> {
    let def = definitions
        .definitions
        .get(&node.definition_id)
        .ok_or_else(|| anyhow::anyhow!("Definition not found: {}", node.definition_id))?;

    // 1. Initialize Context
    // We start with the global workflow state
    let mut ctx_map = workflow_state.context.clone();

    // Inject Node Config into "settings"
    ctx_map.insert(
        "settings".to_string(),
        DataRef::Inline(serde_json::to_value(&node.config)?),
    );

    // Inject Platform Config
    if let Some(platform_id) = &def.meta.platform {
        if let Some(platform) = definitions.platforms.get(platform_id) {
            ctx_map.insert(
                "platform".to_string(),
                DataRef::Inline(serde_json::to_value(&platform.config)?),
            );
        } else {
            // Warn? Or Fail? For now, just log and continue (might use default)
            eprintln!("WARN: Platform definition not found: {}", platform_id);
        }
    }

    // --- NEW: SETUP HANDLEBARS WITH LAZY HELPER ---
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    // Clone store for the closure
    let store_ref = store.cloned();

    handlebars.register_helper(
        "get",
        Box::new(
            move |h: &Helper,
                  _: &Handlebars,
                  ctx: &Context,
                  _: &mut RenderContext,
                  out: &mut dyn Output|
                  -> HelperResult {
                // 1. Get the path string: {{ get "steps.webhook.body.id" }}
                let path = h
                    .param(0)
                    .ok_or(handlebars::RenderErrorReason::Other(
                        "Missing variable path".to_string(),
                    ))?
                    .value()
                    .as_str()
                    .ok_or(handlebars::RenderErrorReason::Other(
                        "Path must be a string".to_string(),
                    ))?;

                // 2. Resolve the root variable from the lightweight context
                let root = ctx.data();
                let parts: Vec<&str> = path.split('.').collect();
                if parts.is_empty() {
                    return Ok(());
                }

                // Find the root DataRef
                let first = parts[0];
                let raw_ref = root.get(first);

                if let Some(data_ref_val) = raw_ref {
                    // Resolve the DataRef (Inline or Blob)
                    let resolved_root = if let Some(obj) = data_ref_val.as_object() {
                        if let Some(blob_ticket_val) = obj.get("Blob") {
                            if let Some(store) = &store_ref {
                                let ticket: crate::store::blob::SecureTicket =
                                    serde_json::from_value(blob_ticket_val.clone()).map_err(
                                        |e| {
                                            handlebars::RenderErrorReason::Other(format!(
                                                "Invalid ticket: {}",
                                                e
                                            ))
                                        },
                                    )?;

                                if let Ok(bytes) = store.claim(&ticket) {
                                    // Attempt to parse as JSON Value
                                    serde_json::from_slice(&bytes).ok()
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else if let Some(inline) = obj.get("Inline") {
                            Some(inline.clone())
                        } else {
                            Some(data_ref_val.clone())
                        }
                    } else {
                        Some(data_ref_val.clone())
                    };

                    if let Some(mut current) = resolved_root {
                        // 3. Traverse remaining parts in the resolved Value
                        for part in &parts[1..] {
                            if let Some(next) = current.get(part) {
                                current = next.clone();
                            } else {
                                // Path not found
                                return Ok(());
                            }
                        }

                        // 4. Render the final value
                        if let Some(s) = current.as_str() {
                            out.write(s)?;
                        } else {
                            out.write(&current.to_string())?;
                        }
                    }
                }
                Ok(())
            },
        ),
    );

    // Also support the explicit `context` variables from definition
    if let Some(ctx_defs) = &def.context {
        for (key, template) in ctx_defs {
            let rendered = handlebars
                .render_template(template, &ctx_map)
                .unwrap_or_else(|_| template.clone());
            // Attempt to parse as JSON if possible, else string
            let val = serde_json::from_str(&rendered).unwrap_or(Value::String(rendered));
            ctx_map.insert(key.clone(), DataRef::Inline(val));
        }
    }

    // "steps" namespace for step outputs
    ctx_map.insert("steps".to_string(), DataRef::Inline(serde_json::json!({})));

    // 2. Execute Steps
    for step in &def.execution {
        let tool = tools
            .get(&step.tool)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", step.tool))?;

        // Resolve Params (Templating)
        // Resolve Params (Templating) & Type Preservation
        let resolved_params = resolve_recursive(&step.params, &ctx_map, &handlebars, store)?;

        // Resolve default mask reference
        let default_masks = std::collections::HashMap::new();
        let masks_ref = shadow_exec
            .map(|s| &s.mocked_tools)
            .unwrap_or(&default_masks);

        // Run Tool
        let mut tool_ctx = ToolContext {
            local: &mut ctx_map,
            memory: global_memory,
            trace_id: trace_id.clone(),
            event_bus: event_bus.clone(),
            shadow_mode: shadow_exec.is_some(),
            shadow_masks: masks_ref,
            secret_store,
            runtime,
        };

        let result = tool.run(&mut tool_ctx, resolved_params)?;

        // Map Returns
        // "returns": { "status": "status_code" } -> context["status_code"] = result["status"]
        if let Some(steps_data) = ctx_map.get_mut("steps")
            && let DataRef::Inline(steps_val) = steps_data
            && let Some(steps_obj) = steps_val.as_object_mut()
        {
            steps_obj.insert(step.id.clone(), result.clone());
        }

        // Returns Mapping
        for (key, var_name) in &step.returns {
            if let Some(val) = result.as_object().and_then(|obj| obj.get(key)) {
                ctx_map.insert(var_name.clone(), DataRef::Inline(val.clone()));
            }
        }
    }

    // 3. Routing (Optional)
    if let Some(routing) = &def.routing {
        // Evaluate Match Expression
        let match_expr_str = &routing.match_expr;
        let resolved_match = handlebars.render_template(match_expr_str, &ctx_map)?;

        // Find matching case
        if let Some(actions) = routing.cases.get(&resolved_match) {
            for action in actions {
                let tool = tools
                    .get(&action.tool)
                    .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", action.tool))?;

                let resolved_params =
                    resolve_recursive(&action.params, &ctx_map, &handlebars, store)?;

                // Resolve default mask reference
                let default_masks = std::collections::HashMap::new();
                let masks_ref = shadow_exec
                    .map(|s| &s.mocked_tools)
                    .unwrap_or(&default_masks);

                let mut tool_ctx = ToolContext {
                    local: &mut ctx_map,
                    memory: global_memory,
                    trace_id: trace_id.clone(),
                    event_bus: event_bus.clone(),
                    shadow_mode: shadow_exec.is_some(),
                    shadow_masks: masks_ref,
                    secret_store,
                    runtime,
                };
                let result = tool.run(&mut tool_ctx, resolved_params)?;

                // Returns Mapping (Routing)
                for (key, var_name) in &action.returns {
                    if let Some(val) = result.as_object().and_then(|obj| obj.get(key)) {
                        ctx_map.insert(var_name.clone(), DataRef::Inline(val.clone()));
                    }
                }
            }
        }
    }

    // 4. Collect Outputs (Emit tool writes to _outputs)

    // 4a. Collect Outputs (Unified Signal / Enriched Bundle logic)
    // We already have `ctx_map` containing step outputs.
    // We merge `_outputs` (explicit emit) into workflow state.
    // BUT we also support `output_transform` which generates new outputs from context.

    // 5. Output Transform
    if let Some(transform_map) = &def.output_transform {
        // Run JMESPath against current context (including step outputs)
        // Materialize full context for JMESPath (expensive but necessary if using transforms)
        let mut materialized = serde_json::Map::new();
        for (k, v) in &ctx_map {
            if let Some(val) = resolve_dataref_to_value(v, store) {
                materialized.insert(k.clone(), val);
            }
        }
        let context_json = Value::Object(materialized);

        for (out_key, expr_str) in transform_map {
            // Compile and search
            if let Ok(expr) = jmespath::compile(expr_str)
                && let Ok(search_res) = expr.search(&context_json)
            {
                let val: Value = serde_json::to_value(&search_res).unwrap_or(Value::Null);
                workflow_state.set(out_key, val);
            }
        }
    }

    // Merge `_outputs` (from explicit Emit tools)
    let outputs = ctx_map
        .get("_outputs")
        .and_then(|d| d.as_inline())
        .cloned()
        .unwrap_or(serde_json::json!({}));
    workflow_state.merge(outputs.clone());

    // 4b. Sync Context back to persistent state
    // We remove node-local settings/platform/steps before persisting to avoid bloat
    ctx_map.remove("settings");
    ctx_map.remove("platform");
    ctx_map.remove("steps");
    workflow_state.context = ctx_map;

    // Determine Active Ports
    let mut active_ports = Vec::new();
    if let Some(out_obj) = outputs.as_object() {
        for key in out_obj.keys() {
            // Treat all emitted keys as potential ports
            active_ports.push(key.clone());
        }
    }

    // Default Fallback
    if active_ports.is_empty() {
        // Check if "Exec" is defined
        let has_exec = def.interface.outputs.iter().any(|p| p.name == "Exec");
        if has_exec {
            active_ports.push("Exec".to_string());
        } else {
            active_ports.push("default".to_string());
        }
    }

    // Emit Node Completion Event
    // This allows UI/Trace to see the final outputs of this node execution
    if let Some(bus) = &event_bus {
        let _ = bus.0.send(crate::api::events::SystemEvent::NodeTelemetry {
            trace_id: trace_id.clone(),
            node_id: node_config.map(|c| c.id).unwrap_or_default(),
            node_type: def.meta.name.clone(),
            execution_ms: 0, // TODO: timer
            success: true,
            details: serde_json::json!({
                "outputs": outputs,
                "active_ports": active_ports
            }),
        });
    }

    Ok(active_ports)
}

fn resolve_recursive(
    value: &Value,
    ctx: &HashMap<String, DataRef>,
    reg: &Handlebars,
    store: Option<&crate::store::BlobStore>,
) -> Result<Value> {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                let inner = &trimmed[2..trimmed.len() - 2].trim();
                if !inner.contains(' ')
                    && !inner.starts_with("get ")
                    && let Some(val) = lookup_path(ctx, inner, store)
                {
                    return Ok(val);
                }
            }
            // Render template using lightweight context
            let rendered = reg.render_template(s, ctx)?;
            Ok(Value::String(rendered))
        }
        Value::Array(arr) => {
            let mut new_arr = Vec::new();
            for v in arr {
                new_arr.push(resolve_recursive(v, ctx, reg, store)?);
            }
            Ok(Value::Array(new_arr))
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), resolve_recursive(v, ctx, reg, store)?);
            }
            Ok(Value::Object(new_obj))
        }
        _ => Ok(value.clone()),
    }
}

fn lookup_path(
    ctx: &HashMap<String, DataRef>,
    path: &str,
    store: Option<&crate::store::BlobStore>,
) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    // Resolve first part from manifest
    let root_ref = ctx.get(parts[0])?;
    let mut current = resolve_dataref_to_value(root_ref, store)?;

    for part in &parts[1..] {
        match current {
            Value::Object(map) => {
                current = map.get(*part)?.clone();
            }
            Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    current = arr.get(idx)?.clone();
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

fn resolve_dataref_to_value(
    data: &DataRef,
    store: Option<&crate::store::BlobStore>,
) -> Option<Value> {
    match data {
        DataRef::Inline(v) => Some(v.clone()),
        DataRef::Blob(ticket) => {
            if let Some(store) = store {
                if let Ok(bytes) = store.claim(ticket) {
                    serde_json::from_slice(&bytes).ok()
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
