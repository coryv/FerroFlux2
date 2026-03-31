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
) -> Result<Vec<String>> {
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

    // 2. Platform injection -- always Inline (loaded from the definitions registry).
    let mut platform_root = serde_json::Map::new();
    for (id, platform) in &definitions.platforms {
        let platform_val = serde_json::to_value(&platform.config).unwrap_or(serde_json::json!({}));
        platform_root.insert(id.clone(), platform_val.clone());

        // Flatten the active platform's keys into the root context.
        if Some(id) == def.meta.platform.as_ref() {
            if let Some(obj) = platform_val.as_object() {
                for (rk, rv) in obj {
                    ctx_map.insert(rk.clone(), DataRef::Inline(rv.clone()));
                }
            }
        }
    }
    ctx_map.insert("platform".to_string(), DataRef::Inline(Value::Object(platform_root)));

    // 3. Within-node Blob cache -- shared across all resolve_recursive calls so
    //    each Blob is claimed from the store at most once per node execution.
    let mut blob_cache: HashMap<String, Value> = HashMap::new();

    // 4. Resolve Node Inputs (fully hydrated for tool use).
    let mut resolved_inputs = serde_json::Map::new();
    for (k, v) in &node.config {
        let result = resolve_recursive(
            v,
            &mut LazyCtx { data: &ctx_map, store, cache: &mut blob_cache },
        )?;
        resolved_inputs.insert(k.clone(), result);
    }
    let inputs_val = Value::Object(resolved_inputs);
    ctx_map.insert("inputs".to_string(), DataRef::Inline(inputs_val.clone()));
    // Pre-populate cache so steps don't need to re-clone from DataRef.
    blob_cache.insert("inputs".to_string(), inputs_val);

    // 5. Resolve explicit context templates.
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

    let mut active_ports = vec!["_next".to_string()];
    let execution_start = Instant::now();

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
            event_bus: event_bus.clone(),
            shadow_mode: shadow_exec.is_some(),
            shadow_masks: masks_ref,
            store,
            secrets: secrets_resolver.as_ref().map(|r| r as &dyn crate::tools::SecretResolver),
        };

        let result = tool.run(&mut tool_ctx, resolved_params)?;

        // Map step output into the "steps" namespace. Collect root promotions
        // separately to avoid holding the steps_val borrow during insertion.
        let mut root_updates: Vec<(String, Value)> = Vec::new();

        if let Some(DataRef::Inline(steps_val)) = ctx_map.get_mut("steps") {
            if let Some(steps_obj) = steps_val.as_object_mut() {
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
        }

        // Promote mapped vars to root context and invalidate their cache entries.
        for (var_name, val) in root_updates {
            ctx_map.insert(var_name.clone(), DataRef::Inline(val));
            blob_cache.remove(&var_name);
        }

        // Invalidate steps cache so the next step picks up the updated object.
        blob_cache.remove("steps");
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
                    event_bus: event_bus.clone(),
                    shadow_mode: shadow_exec.is_some(),
                    shadow_masks: &std::collections::HashMap::new(),
                    store,
                    secrets: None,
                };

                let result = tool.run(&mut tool_ctx, resolved_params)?;
                if action.tool == "emit" {
                    if let Some(port) = result.get("port").and_then(|v| v.as_str()) {
                        active_ports.push(port.to_string());
                    }
                }
            }
        }
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
                "active_ports": active_ports
            }),
        });
    }

    Ok(active_ports)
}
