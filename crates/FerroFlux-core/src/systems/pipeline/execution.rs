use crate::components::execution_state::{ActiveWorkflowState, DataRef};
use crate::components::pipeline::PipelineNode;
use crate::resources::DefinitionRegistry;
use crate::tools::ToolContext;
use crate::tools::ToolRegistry;
use anyhow::Result;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use jmespath;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use super::resolution::{resolve_dataref_to_value, resolve_recursive};

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
    refresh_locks: Option<&ferroflux_db::oauth2::TokenRefreshLocks>,
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
                            // NEW: If the current value is an ActiveWorkflowState, look inside its context
                            if let Some(ctx) = current.get("context") && let Some(val) = ctx.get(part) {
                                current = val.clone();
                            } else if let Some(next) = current.get(part) {
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

    // Materialize context for Handlebars (Resolves DataRefs)
    let mut materialized = serde_json::Map::new();
    for (k, v) in &ctx_map {
        if let Some(val) = resolve_dataref_to_value(v, store) {
            materialized.insert(k.clone(), val);
        }
    }
    
    // Resolve and Inject Inputs from Node Config
    let mut resolved_inputs = serde_json::Map::new();
    let h_ctx_temp = Value::Object(materialized.clone());
    for (k, v) in &node.config {
        let res = resolve_recursive(&v, &h_ctx_temp, &handlebars, store)?;
        resolved_inputs.insert(k.clone(), res);
    }
    
    let inputs_val = Value::Object(resolved_inputs);
    ctx_map.insert("inputs".to_string(), DataRef::Inline(inputs_val.clone()));
    materialized.insert("inputs".to_string(), inputs_val);

    let mut h_ctx = Value::Object(materialized);

    // Inject and Pre-render Platform Config
    if let Some(platform_id) = &def.meta.platform {
        if let Some(platform) = definitions.platforms.get(platform_id) {
            let platform_val = serde_json::to_value(&platform.config).unwrap_or(serde_json::json!({}));
            
            // Re-render platform config with current context (materialized)
            let rendered_platform = resolve_recursive(&platform_val, &h_ctx, &handlebars, store).unwrap_or(platform_val);
            
            ctx_map.insert(
                "platform".to_string(),
                DataRef::Inline(rendered_platform.clone()),
            );
            
            // Update h_ctx for subsequent renderings
            if let Some(obj) = h_ctx.as_object_mut() {
                obj.insert("platform".to_string(), rendered_platform);
            }
        } else {
            eprintln!("WARN: Platform definition not found: {}", platform_id);
        }
    }

    // Also support the explicit `context` variables from definition
    if let Some(ctx_defs) = &def.context {
        for (key, template) in ctx_defs {
            let rendered = handlebars
                .render_template(template, &h_ctx)
                .unwrap_or_else(|_| template.clone());
            // Attempt to parse as JSON if possible, else string
            let val = serde_json::from_str(&rendered).unwrap_or(Value::String(rendered));
            ctx_map.insert(key.clone(), DataRef::Inline(val));
        }
    }

    // "steps" namespace for step outputs
    ctx_map.insert("steps".to_string(), DataRef::Inline(serde_json::json!({})));

    // 2. Execute Steps
    let execution_start = Instant::now();
    for step in &def.execution {
        let tool = tools
            .get(&step.tool)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", step.tool))?;

        // Render template using h_ctx (materialized)
        let resolved_params = if let Some(s) = step.params.as_str() {
             let rendered = handlebars.render_template(s, &h_ctx)?;
             resolve_recursive(&serde_json::from_str(&rendered).unwrap_or(Value::String(rendered)), &h_ctx, &handlebars, store)?
        } else {
             resolve_recursive(&step.params, &h_ctx, &handlebars, store)?
        };

        // Resolve default mask reference
        let default_masks = std::collections::HashMap::new();
        let masks_ref = shadow_exec
            .map(|s| &s.mocked_tools)
            .unwrap_or(&default_masks);

        // Run Tool
        let secrets_resolver = if let Some(ss) = secret_store
            && let Some(rt) = runtime
        {
            Some(crate::tools::CoreSecretResolver {
                tenant_id: ferroflux_types::tenant::TenantId::from("default_tenant"),
                store: ss,
                runtime: rt,
                refresh_locks,
            })
        } else {
            None
        };

        let mut tool_ctx = ToolContext {
            local: &mut ctx_map,
            memory: global_memory,
            trace_id: trace_id.clone(),
            event_bus: event_bus.clone(),
            shadow_mode: shadow_exec.is_some(),
            shadow_masks: masks_ref,
            store,
            secrets: secrets_resolver.as_ref().map(|r| r as &dyn crate::tools::SecretResolver),
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

        // --- NEW: Update h_ctx for the next step ---
        let mut next_materialized = serde_json::Map::new();
        for (k, v) in &ctx_map {
            if let Some(val) = resolve_dataref_to_value(v, store) {
                next_materialized.insert(k.clone(), val);
            }
        }
        h_ctx = Value::Object(next_materialized);
    }

    // 3. Routing (Optional)
    if let Some(routing) = &def.routing {
        // Evaluate Match Expression
        let match_expr_str = &routing.match_expr;
        let resolved_match = handlebars.render_template(match_expr_str, &h_ctx)?;

        // Find matching case
        if let Some(actions) = routing.cases.get(&resolved_match) {
            for action in actions {
                let tool = tools
                    .get(&action.tool)
                    .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", action.tool))?;

                let resolved_params =
                    resolve_recursive(&action.params, &h_ctx, &handlebars, store)?;

                // Resolve default mask reference
                let default_masks = std::collections::HashMap::new();
                let masks_ref = shadow_exec
                    .map(|s| &s.mocked_tools)
                    .unwrap_or(&default_masks);

                let secrets_resolver = if let Some(ss) = secret_store
                    && let Some(rt) = runtime
                {
                    Some(crate::tools::CoreSecretResolver {
                        tenant_id: ferroflux_types::tenant::TenantId::from("default_tenant"),
                        store: ss,
                        runtime: rt,
                        refresh_locks,
                    })
                } else {
                    None
                };

                let mut tool_ctx = ToolContext {
                    local: &mut ctx_map,
                    memory: global_memory,
                    trace_id: trace_id.clone(),
                    event_bus: event_bus.clone(),
                    shadow_mode: shadow_exec.is_some(),
                    shadow_masks: masks_ref,
                    store,
                    secrets: secrets_resolver.as_ref().map(|r| r as &dyn crate::tools::SecretResolver),
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
                ctx_map.insert(out_key.clone(), DataRef::Inline(val));
            }
        }
    }

    let outputs = ctx_map
        .get("_outputs")
        .and_then(|d| d.as_inline())
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // Merge outputs into ctx_map
    if let Some(obj) = outputs.as_object() {
        for (k, v) in obj {
            ctx_map.insert(k.clone(), DataRef::Inline(v.clone()));
        }
    }

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
            execution_ms: execution_start.elapsed().as_millis() as u64,
            success: true,
            details: serde_json::json!({
                "outputs": outputs,
                "active_ports": active_ports
            }),
        });
    }

    Ok(active_ports)
}
