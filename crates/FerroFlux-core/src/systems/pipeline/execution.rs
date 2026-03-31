use crate::components::execution_state::{ActiveWorkflowState, DataRef};
use crate::components::pipeline::PipelineNode;
use crate::resources::DefinitionRegistry;
use crate::tools::ToolContext;
use crate::tools::ToolRegistry;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use super::resolution::{resolve_dataref_to_value, resolve_recursive};

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
) -> Result<Vec<String>> {
    let def = definitions
        .definitions
        .get(&node.definition_id)
        .ok_or_else(|| anyhow::anyhow!("Definition not found: {}", node.definition_id))?;

    tracing::trace!(node_id = %node.definition_id, "Executing pipeline node");

    // 1. Initialize Context Map
    let mut ctx_map = workflow_state.context.clone();
    
    // Inject Settings
    ctx_map.insert("settings".to_string(), DataRef::Inline(serde_json::to_value(&node.config)?));

    // Materialize initial context for rendering
    let mut materialized = serde_json::Map::new();
    for (k, v) in &ctx_map {
        if let Some(val) = resolve_dataref_to_value(v, store) {
            materialized.insert(k.clone(), val);
        }
    }

    // 2. Platform Injection - Merge into root materialized map FIRST
    let mut platform_root = serde_json::Map::new();
    for (id, platform) in &definitions.platforms {
        let platform_val = serde_json::to_value(&platform.config).unwrap_or(serde_json::json!({}));
        platform_root.insert(id.clone(), platform_val.clone());
        
        // If this is the active platform for the node, merge its keys into root context
        if Some(id) == def.meta.platform.as_ref() && let Some(obj) = platform_val.as_object() {
            for (rk, rv) in obj {
                materialized.insert(rk.clone(), rv.clone());
            }
        }
    }
    materialized.insert("platform".to_string(), Value::Object(platform_root));
    materialized.insert("trace_id".to_string(), Value::String(trace_id.clone()));

    let mut h_ctx = Value::Object(materialized);

    // 3. Resolve Node Inputs (Fully Hydrated for Tool Use)
    let mut resolved_inputs = serde_json::Map::new();
    for (k, v) in &node.config {
        let res = resolve_recursive(v, &h_ctx, &None, store)?;
        resolved_inputs.insert(k.clone(), res);
    }
    let inputs_val = Value::Object(resolved_inputs.clone());
    
    // Inject inputs into context for steps to see
    if let Some(obj) = h_ctx.as_object_mut() {
        obj.insert("inputs".to_string(), inputs_val.clone());
    }
    ctx_map.insert("inputs".to_string(), DataRef::Inline(inputs_val));

    // 4. Resolve explicit context
    if let Some(ctx_defs) = &def.context {
        for (key, template) in ctx_defs {
            let val = resolve_recursive(&Value::String(template.clone()), &h_ctx, &None, store)?;
            ctx_map.insert(key.clone(), DataRef::Inline(val.clone()));
            if let Some(obj) = h_ctx.as_object_mut() {
                obj.insert(key.clone(), val);
            }
        }
    }

    // 5. Steps Execution
    ctx_map.insert("steps".to_string(), DataRef::Inline(serde_json::json!({})));
    if let Some(obj) = h_ctx.as_object_mut() {
        obj.insert("steps".to_string(), serde_json::json!({}));
    }

    let mut active_ports = vec!["_next".to_string()];
    let execution_start = Instant::now();

    for step in &def.execution {
        let tool = tools.get(&step.tool).ok_or_else(|| anyhow::anyhow!("Tool not found: {}", step.tool))?;
        let resolved_params = resolve_recursive(&step.params, &h_ctx, &None, store)?;

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
            event_bus: event_bus.clone(),
            shadow_mode: shadow_exec.is_some(),
            shadow_masks: masks_ref,
            store,
            secrets: secrets_resolver.as_ref().map(|r| r as &dyn crate::tools::SecretResolver),
        };

        let result = tool.run(&mut tool_ctx, resolved_params)?;

        // Map step output into "steps" namespace
        let mut root_updates = Vec::new();
        if let Some(steps_data) = ctx_map.get_mut("steps") && let DataRef::Inline(steps_val) = steps_data && let Some(steps_obj) = steps_val.as_object_mut() {
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
            
            // Update h_ctx for next steps
            if let Some(h_obj) = h_ctx.as_object_mut() {
                h_obj.insert("steps".to_string(), Value::Object(steps_obj.clone()));
            }
        }

        // Apply root context updates
        for (var_name, val) in root_updates {
            ctx_map.insert(var_name.clone(), DataRef::Inline(val.clone()));
            if let Some(h_obj) = h_ctx.as_object_mut() {
                h_obj.insert(var_name, val);
            }
        }
    }

    // 6. Routing
    if let Some(routing) = &def.routing {
        let res = resolve_recursive(&Value::String(routing.match_expr.clone()), &h_ctx, &None, store)?;
        let resolved_match = res.as_str().unwrap_or("").to_string();
        
        if let Some(actions) = routing.cases.get(&resolved_match) {
            for action in actions {
                let tool = tools.get(&action.tool).ok_or_else(|| anyhow::anyhow!("Tool not found: {}", action.tool))?;
                let resolved_params = resolve_recursive(&action.params, &h_ctx, &None, store)?;
                
                let mut tool_ctx = ToolContext {
                    local: &mut ctx_map,
                    memory: global_memory,
                    trace_id: trace_id.clone(),
                    event_bus: event_bus.clone(),
                    shadow_mode: shadow_exec.is_some(),
                    shadow_masks: &std::collections::HashMap::new(),
                    store,
                    secrets: None,
                };
                
                let result = tool.run(&mut tool_ctx, resolved_params)?;
                if action.tool == "emit" && let Some(port) = result.get("port").and_then(|v| v.as_str()) {
                    active_ports.push(port.to_string());
                }
            }
        }
    }

    // 7. Telemetry
    if let Some(bus) = &event_bus {
        let _ = bus.0.send(crate::api::events::SystemEvent::NodeTelemetry {
            trace_id: trace_id.clone(),
            node_id: uuid::Uuid::new_v4(), // Generic ID for telemetry record
            node_type: "Pipeline".to_string(),
            execution_ms: execution_start.elapsed().as_millis() as u64,
            success: true,
            details: serde_json::json!({
                "definition_id": node.definition_id,
                "active_ports": active_ports
            }),
        });
    }

    Ok(active_ports)
}
