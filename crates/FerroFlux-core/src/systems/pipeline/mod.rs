use crate::components::execution_state::ActiveWorkflowState;
use crate::components::pipeline::{PipelineNode, InputBuffer};
use crate::resources::{DefinitionRegistry, GraphTopology, WorkDone, TokioRuntime};
use crate::tools::ToolRegistry;
use bevy_ecs::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

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
        Option<&mut InputBuffer>,
    )>,
    node_registry: Res<DefinitionRegistry>,
    tool_registry: Res<ToolRegistry>,
    store: Res<crate::store::BlobStore>,
    bus: Res<crate::api::events::SystemEventBus>,
    secret_store: Res<crate::secrets::DatabaseSecretStore>,
    runtime: Res<TokioRuntime>,
    refresh_locks: Res<ferroflux_db::oauth2::TokenRefreshLocks>,
    mut work_done: ResMut<WorkDone>,
    topology: Res<GraphTopology>,
    mut commands: Commands,
    mut global_memory: ResMut<crate::resources::GlobalMemory>,
    workflow_defs: Query<&crate::components::execution_state::WorkflowDefinition>,
) {
    let query_count = query.iter().count();
    if query_count > 0 {
        tracing::debug!(count = query_count, "pipeline_execution_system: running query");
    }

    for (entity, mut node, mut inbox, mut outbox, shadow_exec, node_config, mut input_buffer) in
        query.iter_mut()
    {
        // 1. Ingest all pending tickets into the InputBuffer
        let mut temp_new_buffer = None;
        while let Some((target_port, ticket)) = inbox.queue.pop_front() {
            let port_name = target_port.clone().unwrap_or_else(|| "Exec".to_string());
            let trace_id_str = ticket.metadata.get("trace_id").cloned().unwrap_or_default();
            if let Ok(trace_id) = Uuid::parse_str(&trace_id_str) {
                if let Some(ref mut buffer) = input_buffer {
                    buffer.traces.entry(trace_id).or_default().insert(port_name, ticket);
                } else {
                    let buffer = temp_new_buffer.get_or_insert_with(InputBuffer::default);
                    buffer.traces.entry(trace_id).or_default().insert(port_name, ticket);
                }
            }
            work_done.0 = true;
        }

        if let Some(new_buffer) = temp_new_buffer {
            commands.entity(entity).insert(new_buffer);
        }

        // 2. Process ready traces
        // Note: If we just inserted InputBuffer via commands, it won't be available in the query until next frame.
        // We'll handle both cases if possible, but Bevy's Query approach usually means waiting a frame.
        // However, if it ALREADY has a buffer, we process it now.
        if let Some(ref mut buffer) = input_buffer {
            let mut ready_traces = Vec::new();

            // Get node definition to find required ports
            let def = match node_registry.definitions.get(&node.definition_id) {
                Some(d) => d,
                None => continue,
            };

            // Required ports: ports with inbound connections + "Exec" if it exists
            let inbound_ports = topology.inbound_connections.get(&entity);
            let has_exec_input = def.interface.inputs.iter().any(|i| i.name == "Exec");

            for (trace_id, port_tickets) in &buffer.traces {
                let mut is_ready = true;

                // Sync Rule: Must have "Exec" if defined in the interface
                if has_exec_input && !port_tickets.contains_key("Exec") {
                    is_ready = false;
                }

                // Sync Rule: Must have all inbound ports that are connected to other nodes
                if let Some(required) = inbound_ports {
                    for req_port in required {
                        if !port_tickets.contains_key(req_port) {
                            is_ready = false;
                            break;
                        }
                    }
                }

                if is_ready {
                    ready_traces.push(*trace_id);
                }
            }

            for trace_id in ready_traces {
                if let Some(port_tickets) = buffer.traces.remove(&trace_id) {
                    tracing::info!(entity = ?entity, trace_id = %trace_id, "Synchronization Gate: All inputs present, executing");
                    work_done.0 = true;

                    // 1. Merge States
                    // We pick one ticket as the "Primary" (prioritize Exec) and merge others into it.
                    let primary_ticket = port_tickets
                        .get("Exec")
                        .or_else(|| port_tickets.values().next())
                        .unwrap();
                    
                    let data = match store.claim(primary_ticket) {
                        Ok(d) => d,
                        Err(e) => {
                            tracing::error!("Failed to claim primary ticket: {}", e);
                            continue;
                        }
                    };

                    let mut state: ActiveWorkflowState = if let Ok(s) = serde_json::from_slice(&data) {
                        s
                    } else {
                        let mut s = ActiveWorkflowState::new();
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&data) {
                            if let Some(ticket_port) = port_tickets.iter().find(|(_, t)| t.id == primary_ticket.id).map(|(p, _)| p) {
                                let mut map = serde_json::Map::new();
                                map.insert(ticket_port.clone(), val);
                                s.merge(serde_json::Value::Object(map));
                            } else {
                                s.merge(val);
                            }
                        }
                        s
                    };

                    // Merge all tickets into the state
                    for (port_name, ticket) in &port_tickets {
                        if let Ok(ticket_data) = store.claim(ticket) {
                            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&ticket_data) {
                                // If it's a state, merge its context
                                if let Some(obj) = val.as_object() && obj.contains_key("context") && obj.contains_key("history")
                                    && let Ok(incoming_state) = serde_json::from_value::<ActiveWorkflowState>(val.clone())
                                {
                                    for (k, v) in &incoming_state.context {
                                        state.set_ref(&format!("{}.{}", port_name, k), v.clone());
                                        if port_name == "body" || port_name == "Exec" {
                                            state.set_ref(k, v.clone());
                                        }
                                    }
                                    if let Some(val_ref) = incoming_state.context.get("_value") {
                                        state.set_ref(port_name, val_ref.clone());
                                    }
                                } else {
                                    state.set(port_name, val);
                                }
                            }
                        }
                    }

                    // 2. Execute
                    let mut memory_lock = global_memory.0.lock().expect("GlobalMemory poisoned");
                    let t_id = trace_id.to_string();
                    
                    let emissions = tokio::task::block_in_place(|| {
                        execute_pipeline_node(
                            &mut node,
                            &mut state,
                            &node_registry,
                            &tool_registry,
                            &mut *memory_lock,
                            t_id.clone(),
                            Some(bus.clone()),
                            Some(&store),
                            shadow_exec,
                            node_config,
                            Some(&secret_store),
                            Some(&runtime),
                            Some(&refresh_locks),
                            workflow_defs.iter().find(|d| Some(&d.id) == node_config.map(|c| &c.workflow_id)).map(|d| d.config.clone()).unwrap_or_default(),
                        )
                    })
                    .map_err(|e| {
                        tracing::error!(trace_id = %t_id, "Pipeline execution failed: {}", e);
                        e
                    });

                    let emissions = match emissions {
                        Ok(e) => e,
                        Err(e) => {
                            if !t_id.is_empty() {
                                let _ = bus.0.send(crate::api::events::SystemEvent::NodeError {
                                    trace_id: t_id,
                                    node_id: node_config.map(|c| c.id).unwrap_or_default(),
                                    error: e.to_string(),
                                    timestamp: chrono::Utc::now().timestamp_millis(),
                                });
                            }
                            state.set(
                                "_error",
                                serde_json::json!({
                                    "message": e.to_string(),
                                    "node_id": node_config.map(|c| c.id).unwrap_or_default()
                                }),
                            );
                            vec![("_error".to_string(), Value::Null, state)]
                        }
                    };

                    for (port, val, mut out_state) in emissions {
                        if !val.is_null() {
                            out_state.set("_value", val);
                        }
                        if let Ok(new_bytes) = serde_json::to_vec(&out_state) {
                            if let Ok(new_ticket) =
                                store.check_in_with_metadata(&new_bytes, primary_ticket.metadata.clone())
                            {
                                outbox.queue.push_back((Some(port), new_ticket));
                            }
                        }
                    }
                }
            }
        }
    }
}
