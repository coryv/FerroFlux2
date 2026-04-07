use crate::components::execution_state::{ActiveWorkflowState, DataRef};
use crate::components::pipeline::PipelineNode;
use crate::resources::DefinitionRegistry;
use crate::tools::ToolContext;
use crate::tools::ToolRegistry;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use super::resolution::{resolve_recursive, LazyCtx};

/// Helper function to execute a single pipeline node.
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
    _node_config: Option<&crate::components::NodeConfig>,
    secret_store: Option<&crate::secrets::DatabaseSecretStore>,
    runtime: Option<&crate::resources::TokioRuntime>,
    refresh_locks: Option<&ferroflux_db::oauth2::TokenRefreshLocks>,
    workflow_config: HashMap<String, Value>,
) -> Result<Vec<(String, Value, ActiveWorkflowState)>> {
    let def = definitions
        .definitions
        .get(&node.definition_id)
        .ok_or_else(|| anyhow::anyhow!("Definition not found: {}", node.definition_id))?;

    tracing::trace!(node_id = %node.definition_id, "Executing pipeline node");

    // 1. Initialize Context Map from workflow state.
    //    Values remain as DataRef (Inline or Blob) -- no upfront materialization.
    let mut ctx_map = workflow_state.context.clone();

    // Settings are always small/Inline.
    ctx_map.insert("settings".to_string(), DataRef::Inline(serde_json::to_value(&node.config)?));
    ctx_map.insert("trace_id".to_string(), DataRef::Inline(Value::String(trace_id.clone())));
    ctx_map.insert("config".to_string(), DataRef::Inline(Value::Object(workflow_config.clone().into_iter().collect())));

    // Inject 'self' for explicit internal access.
    let mut self_obj = serde_json::Map::new();
    self_obj.insert("id".to_string(), Value::String(node.definition_id.clone()));
    self_obj.insert("settings".to_string(), serde_json::to_value(&node.config).unwrap_or(Value::Null));
    
    if let Some(platform_id) = &def.meta.platform && let Some(p) = definitions.platforms.get(platform_id) {
        self_obj.insert("config".to_string(), serde_json::to_value(&p.config).unwrap_or(Value::Object(serde_json::Map::new())));
    }
    ctx_map.insert("self".to_string(), DataRef::Inline(Value::Object(self_obj)));

    // 2. Platform injection -- always Inline (loaded from the definitions registry).
    let mut platforms_root = serde_json::Map::new();
    let mut active_platform_val = None;

    for (id, platform) in &definitions.platforms {
        let platform_val = serde_json::to_value(&platform.config).unwrap_or(serde_json::json!({}));
        platforms_root.insert(id.clone(), platform_val.clone());

        // Identify and store the active platform's config
        if Some(id) == def.meta.platform.as_ref() {
            active_platform_val = Some(platform_val.clone());
            
            // Flatten the active platform's keys into the root context for convenience (e.g. headers, base_url).
            if let Some(obj) = platform_val.as_object() {
                for (rk, rv) in obj {
                    ctx_map.insert(rk.clone(), DataRef::Inline(rv.clone()));
                }
            }
        }
    }
    
    // Inject all platforms map
    ctx_map.insert("platforms".to_string(), DataRef::Inline(Value::Object(platforms_root)));

    // Inject active platform specifically as 'platform'
    if let Some(apv) = active_platform_val {
        ctx_map.insert("platform".to_string(), DataRef::Inline(apv));
    }

    // 3. Within-node Blob cache -- shared across all resolve_recursive calls so
    //    each Blob is claimed from the store at most once per node execution.
    let mut blob_cache: HashMap<String, Value> = HashMap::new();

    // 4. Resolve Node settings (fully hydrated for tool use).
    let mut resolved_settings = serde_json::Map::new();
    for s in &def.interface.settings {
        resolved_settings.insert(s.name.clone(), Value::Null);
    }
    for (k, v) in &node.config {
        let result = resolve_recursive(
            v,
            &mut LazyCtx { data: &ctx_map, store, cache: &mut blob_cache },
        )?;
        resolved_settings.insert(k.clone(), result);
    }
    let settings_val = Value::Object(resolved_settings.clone());
    ctx_map.insert("settings".to_string(), DataRef::Inline(settings_val.clone()));
    
    // Update self.settings with the resolved values
    if let Some(DataRef::Inline(Value::Object(self_map))) = ctx_map.get_mut("self") {
        self_map.insert("settings".to_string(), settings_val.clone());
    }
    blob_cache.insert("settings".to_string(), settings_val);

    // 5. Populate `inputs` from workflow context (ports).
    let mut inputs_map = serde_json::Map::new();
    let declared_inputs: Vec<String> = def.interface.inputs.iter().map(|i| i.name.clone()).collect();
    
    if declared_inputs.is_empty() {
        // Fallback: Populate with ALL context keys if none are declared (Catch-all mode)
        for k in workflow_state.context.keys() {
            if let Some(val) = (LazyCtx { data: &workflow_state.context, store, cache: &mut blob_cache }).materialize_key(k) {
                inputs_map.insert(k.clone(), val);
            }
        }
    } else {
        // Strict mode: Only populate declared ports, default missing ones to settings then Null
        for name in declared_inputs {
            if let Some(val) = (LazyCtx { data: &workflow_state.context, store, cache: &mut blob_cache }).materialize_key(&name) {
                inputs_map.insert(name, val);
            } else if let Some(val) = resolved_settings.get(&name) {
                // Fallback to RESOLVED configuration for inputs
                inputs_map.insert(name.to_string(), val.clone());
            } else {
                inputs_map.insert(name, Value::Null);
            }
        }
    }
    
    let inputs_val = Value::Object(inputs_map);
    ctx_map.insert("inputs".to_string(), DataRef::Inline(inputs_val.clone()));
    blob_cache.insert("inputs".to_string(), inputs_val);

    // 6. Resolve explicit context templates.
    if let Some(ctx_defs) = &def.context {
        for (key, template) in ctx_defs {
            let val = resolve_recursive(
                &Value::String(template.clone()),
                &mut LazyCtx { data: &ctx_map, store, cache: &mut blob_cache },
            )?;
            ctx_map.insert(key.clone(), DataRef::Inline(val.clone()));
            blob_cache.insert(key.clone(), val);
        }
    }

    // 6. Steps Execution.
    ctx_map.insert("steps".to_string(), DataRef::Inline(serde_json::json!({})));
    blob_cache.insert("steps".to_string(), serde_json::json!({}));

    let mut emissions: Vec<(String, Value, ActiveWorkflowState)> = Vec::new();
    let execution_start = Instant::now();

    let is_iterator = def.meta.node_subtype.as_deref() == Some("Iterator");
    let mut iteration_count = 0;
    const MAX_ITERATIONS: usize = 1000;

    loop {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            tracing::warn!(node_id = %node.definition_id, "Max iterations reached (1000) - breaking jump to avoid infinite loop");
            break;
        }

        let mut iteration_done = true; // Assume done unless an iterator tool says otherwise

        for step in &def.execution {
            let tool = tools.get(&step.tool)
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", step.tool))?;

            let resolved_params = resolve_recursive(
                &step.params,
                &mut LazyCtx { data: &ctx_map, store, cache: &mut blob_cache },
            )?;

            let default_masks = std::collections::HashMap::new();
            let masks_ref = shadow_exec.map(|s| &s.mocked_tools).unwrap_or(&default_masks);

            let secrets_resolver = if let Some(ss) = secret_store && let Some(rt) = runtime {
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
                node_id: node.definition_id.clone(),
                tenant_id: "default_tenant".to_string(),
                event_bus: event_bus.clone(),
                shadow_mode: shadow_exec.is_some(),
                shadow_masks: masks_ref,
                store,
                secrets: secrets_resolver.as_ref().map(|r| r as &dyn crate::tools::SecretResolver),
            };

            let result = tool.run(&mut tool_ctx, resolved_params)?;

            // If the tool is 'split', it may signal if we are done or not
            if step.tool == "split" && let Some(done) = result.get("is_done").and_then(|v| v.as_bool()) {
                iteration_done = done;
            }

            // If the tool is 'emit', capture current state snapshot and the value for this port
            if step.tool == "emit" && let Some(port) = result.get("port").and_then(|v| v.as_str()) {
                let val = result.get("value").cloned().unwrap_or(Value::Null);
                // Take a snapshot of the current workflow state (context + history)
                let mut state_snapshot = workflow_state.clone();

                // Inject the node's output so downstream nodes can reference it directly via its ID
                let mut node_data = serde_json::Map::new();
                if let Some(DataRef::Inline(Value::Object(existing))) = state_snapshot.context.get(&node.definition_id) {
                    node_data = existing.clone();
                }
                node_data.insert(port.to_string(), val.clone());
                state_snapshot.set_ref(&node.definition_id, DataRef::Inline(Value::Object(node_data)));

                for (k, v) in &ctx_map {
                    if k != "inputs" && k != "steps" && k != "settings" && k != "platform" {
                        state_snapshot.set_ref(k, v.clone());
                    }
                }
                emissions.push((port.to_string(), val, state_snapshot));
            }

            // Map step output into the "steps" namespace...
            let mut root_updates: Vec<(String, Value)> = Vec::new();
            if let Some(DataRef::Inline(steps_val)) = ctx_map.get_mut("steps")
                && let Some(steps_obj) = steps_val.as_object_mut()
            {
                if step.returns.is_empty() {
                    steps_obj.insert(step.id.clone(), result.clone());
                } else {
                    let mut step_out = serde_json::Map::new();
                    for (key, var_name) in &step.returns {
                        if let Some(val) = result.get(key) {
                            step_out.insert(var_name.clone(), val.clone());
                            root_updates.push((var_name.clone(), val.clone()));
                        }
                    }
                    steps_obj.insert(step.id.clone(), Value::Object(step_out));
                }
            }

            for (var_name, val) in root_updates {
                ctx_map.insert(var_name.clone(), DataRef::Inline(val));
                blob_cache.remove(&var_name);
            }
            blob_cache.remove("steps");
        }

        if !is_iterator || iteration_done {
            break;
        }
    }

    // 7. Routing.
    if let Some(routing) = &def.routing {
        let res = resolve_recursive(
            &Value::String(routing.match_expr.clone()),
            &mut LazyCtx { data: &ctx_map, store, cache: &mut blob_cache },
        )?;
        let resolved_match = res.as_str().unwrap_or("").to_string();

        if let Some(actions) = routing.cases.get(&resolved_match)
            .or_else(|| routing.cases.get("default"))
        {
            for action in actions {
                let tool = tools.get(&action.tool)
                    .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", action.tool))?;
                let resolved_params = resolve_recursive(
                    &action.params,
                    &mut LazyCtx { data: &ctx_map, store, cache: &mut blob_cache },
                )?;

                let mut tool_ctx = ToolContext {
                    local: &mut ctx_map,
                    memory: global_memory,
                    trace_id: trace_id.clone(),
                    node_id: node.definition_id.clone(),
                    tenant_id: "default_tenant".to_string(),
                    event_bus: event_bus.clone(),
                    shadow_mode: shadow_exec.is_some(),
                    shadow_masks: &std::collections::HashMap::new(),
                    store,
                    secrets: None,
                };

                let result = tool.run(&mut tool_ctx, resolved_params)?;
                if action.tool == "emit"
                    && let Some(port) = result.get("port").and_then(|v| v.as_str())
                {
                    let val = result.get("value").cloned().unwrap_or(Value::Null);
                    let mut state_snapshot = workflow_state.clone();

                    let mut node_data = serde_json::Map::new();
                    if let Some(DataRef::Inline(Value::Object(existing))) = state_snapshot.context.get(&node.definition_id) {
                        node_data = existing.clone();
                    }
                    node_data.insert(port.to_string(), val.clone());
                    state_snapshot.set_ref(&node.definition_id, DataRef::Inline(Value::Object(node_data)));

                    for (k, v) in &ctx_map {
                        if k != "inputs" && k != "steps" && k != "settings" && k != "platform" {
                            state_snapshot.set_ref(k, v.clone());
                        }
                    }
                    emissions.push((port.to_string(), val, state_snapshot));
                }
            }
        }
    }

    // Capture the final state for the default _next port if no emissions occurred
    if emissions.is_empty() {
        let mut final_state = workflow_state.clone();

        let mut node_data = serde_json::Map::new();
        if let Some(DataRef::Inline(Value::Object(existing))) = final_state.context.get(&node.definition_id) {
            node_data = existing.clone();
        }
        node_data.insert("_next".to_string(), Value::Null);
        final_state.set_ref(&node.definition_id, DataRef::Inline(Value::Object(node_data)));

        for (k, v) in &ctx_map {
            if k != "inputs" && k != "steps" && k != "settings" && k != "platform" {
                final_state.set_ref(k, v.clone());
            }
        }
        emissions.push(("_next".to_string(), Value::Null, final_state));
    }

    // 8. Telemetry.
    if let Some(bus) = &event_bus {
        let _ = bus.0.send(crate::api::events::SystemEvent::NodeTelemetry {
            trace_id: trace_id.clone(),
            node_id: uuid::Uuid::new_v4(),
            node_type: "Pipeline".to_string(),
            execution_ms: execution_start.elapsed().as_millis() as u64,
            success: true,
            details: serde_json::json!({
                "definition_id": node.definition_id,
                "emissions_count": emissions.len()
            }),
        });
    }

    Ok(emissions)
}
