use crate::components::execution_state::ActiveWorkflowState;
use crate::components::pipeline::PipelineNode;
use crate::resources::DefinitionRegistry;
use crate::tools::ToolRegistry;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

pub mod execution;
pub mod resolution;

use execution::execute_pipeline_node;

/// System that executes PipelineNodes when triggered.
///
/// NOTE: In a real implementation, this would likely be an async system or spawned task.
/// For this MVP, we execute synchronously when an "Exec" signal is received (implied).
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
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
    refresh_locks: Res<ferroflux_db::oauth2::TokenRefreshLocks>,
) {
    const MAX_ITEMS_PER_TICK: usize = 50;
    for (_entity, mut node, mut inbox, mut outbox, shadow_exec, node_config) in query.iter_mut() {
        let mut processed_count = 0;
        while processed_count < MAX_ITEMS_PER_TICK {
            let (target_port, ticket) = match inbox.queue.pop_front() {
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
                        // If it came in on a specific port, put it there!
                        if let Some(ref p) = target_port {
                            let mut map = serde_json::Map::new();
                            map.insert(p.clone(), val);
                            s.merge(serde_json::Value::Object(map));
                        } else {
                            s.merge(val);
                        }
                    }
                    s
                };

                // Explicitly map the current ticket's data to the target_port in the state
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&data) {
                    // Special Case: If the value looks like an ActiveWorkflowState, merge its context
                    if let Some(obj) = val.as_object() && obj.contains_key("context") && obj.contains_key("history") {
                         if let Ok(incoming_state) = serde_json::from_value::<ActiveWorkflowState>(val.clone()) {
                             for (k, v) in incoming_state.context {
                                 if let Some(ref p) = target_port {
                                     state.set_ref(&format!("{}.{}", p, k), v.clone());
                                     if p == "body" {
                                         state.set_ref(&k, v);
                                     }
                                 } else {
                                     state.set_ref(&k, v);
                                 }
                             }
                         }
                    } else if let Some(ref p) = target_port {
                         state.set(p, val);
                    } else {
                         state.merge(val);
                    }
                }

                // 2. Execute
                let mut memory = HashMap::new(); // Global memory stub
                // 3. Output
                // We use block_in_place because the pipeline execution might call blocking tools
                // (like HttpClientTool) and we are likely running inside a Tokio worker thread
                // during tests or in some production environments.
                let active_ports = tokio::task::block_in_place(|| {
                    execute_pipeline_node(
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
                        Some(&refresh_locks),
                    )
                })
                .map_err(|e| {
                    let t_id = ticket.metadata.get("trace_id").cloned().unwrap_or_default();
                    tracing::error!(trace_id = %t_id, "Pipeline execution failed: {}", e);
                    e
                });

                let t_id = ticket.metadata.get("trace_id").cloned().unwrap_or_default();
                let active_ports = match active_ports {
                    Ok(ports) => ports,
                    Err(e) => {

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
