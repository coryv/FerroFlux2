use crate::components::{Edge, EdgeLabel, Inbox, SwitchConfig, WorkDone};
use crate::store::{BlobStore, SecureTicket};
use bevy_ecs::prelude::*;
use rhai::{Engine, Scope};
use std::str;

// Improved switch worker with ParamSet logic fully implemented
#[tracing::instrument(skip(param_set, edge_query, store, engine, work_done, event_bus))]
#[allow(clippy::type_complexity)]
pub fn switch_worker_safe(
    mut param_set: ParamSet<(
        Query<(
            Entity,
            &SwitchConfig,
            &crate::components::NodeConfig,
            &mut Inbox,
        )>, // Iterate this
        Query<&mut Inbox>, // Apply to this
    )>,
    edge_query: Query<(&Edge, Option<&EdgeLabel>)>,
    store: Res<BlobStore>,
    engine: NonSend<Engine>,
    mut work_done: ResMut<WorkDone>,
    event_bus: Res<crate::api::events::SystemEventBus>,
) {
    let mut actions: Vec<(Entity, SecureTicket)> = Vec::new();
    let event_tx = event_bus.0.clone();

    // 1. Collect Valid Actions
    {
        let mut switches = param_set.p0();
        for (entity, config, node_config, mut inbox) in switches.iter_mut() {
            while let Some(ticket) = inbox.queue.pop_front() {
                work_done.0 = true;
                let start = std::time::Instant::now();

                // Claim & Eval
                let (data, trace_id) = match store.claim(&ticket) {
                    Ok(d) => (
                        d,
                        ticket
                            .metadata
                            .get("trace_id")
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string()),
                    ),
                    Err(_) => continue,
                };

                let data_str = str::from_utf8(&data).unwrap_or("0");
                let mut scope = Scope::new();
                if let Ok(val) = data_str.parse::<f64>() {
                    scope.push("input", val);
                } else {
                    scope.push("input", data_str.to_string());
                }

                // Evaluate as Dynamic to support diverse return types
                let result_dynamic = engine
                    .eval_with_scope::<rhai::Dynamic>(&mut scope, &config.script)
                    .ok();

                let (target_label, decision) = if let Some(val) = result_dynamic {
                    if val.is::<bool>() {
                        let b = val.cast::<bool>();
                        let lbl = if b {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        };
                        (lbl, b.to_string())
                    } else if val.is::<String>() {
                        let s = val.cast::<String>();
                        (s.clone(), s)
                    } else if let Ok(s) = val.clone().into_string() {
                        // Fallback: try to convert to string (e.g. integer)
                        (s.clone(), s)
                    } else {
                        tracing::warn!(node_id = %node_config.id, return_type = ?val, "Script returned unsupported type");
                        continue;
                    }
                } else {
                    tracing::error!(node_id = %node_config.id, "Script evaluation failed");
                    continue;
                };

                // Find Target
                let mut routed = false;
                for (edge, label) in edge_query.iter() {
                    if edge.source == entity
                        && let Some(lbl) = label
                        && lbl.0 == target_label
                    {
                        actions.push((edge.target, ticket.clone()));
                        tracing::info!(node_id = %node_config.id, target_label = %target_label, "Switch routed");
                        routed = true;
                        break;
                    }
                }

                // Telemetry
                let elapsed = start.elapsed().as_millis() as u64;
                let _ = event_tx.send(crate::api::events::SystemEvent::NodeTelemetry {
                    trace_id,
                    node_id: node_config.id,
                    node_type: "Switch".to_string(),
                    execution_ms: elapsed,
                    success: routed,
                    details: serde_json::json!({
                        "decision": decision,
                        "routed": routed
                    }),
                });
            }
        }
    }

    // 2. Apply Actions
    let mut inboxes = param_set.p1();
    for (target, ticket) in actions {
        if let Ok(mut target_inbox) = inboxes.get_mut(target) {
            target_inbox.queue.push_back(ticket);
        }
    }
}

#[tracing::instrument(skip(query, store, engine, work_done, event_bus))]
pub fn script_worker(
    mut query: Query<(
        &crate::components::ScriptConfig,
        &crate::components::NodeConfig,
        &mut Inbox,
        &mut crate::components::Outbox,
    )>,
    store: Res<BlobStore>,
    engine: NonSend<Engine>,
    mut work_done: ResMut<WorkDone>,
    event_bus: Res<crate::api::events::SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            work_done.0 = true;
            let start = std::time::Instant::now();

            let (data, trace_id) = match store.claim(&ticket) {
                Ok(d) => (
                    d,
                    ticket
                        .metadata
                        .get("trace_id")
                        .cloned()
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                ),
                Err(_) => continue,
            };
            let data_str = str::from_utf8(&data).unwrap_or("");

            // Eval
            // Minimal implementation: just eval and return result
            // Advanced: Pass input as variable
            let mut scope = Scope::new();
            scope.push("input", data_str.to_string());

            // Parse input for optional merging
            let input_val: serde_json::Value =
                serde_json::from_str(data_str).unwrap_or(serde_json::json!({}));

            let result = engine.eval_with_scope::<String>(&mut scope, &config.script);

            let (output_str, success) = match result {
                Ok(s) => (s, true),
                Err(e) => (format!("ErrorFull: {}", e), false),
            };

            // MERGE RESULT
            let output = crate::systems::utils::merge_result(
                &input_val,
                &output_str,
                config.result_key.as_ref(),
            );

            let mut out_meta = std::collections::HashMap::new();
            out_meta.insert("trace_id".to_string(), trace_id.clone());

            if let Ok(out_ticket) = store.check_in_with_metadata(output.as_bytes(), out_meta) {
                outbox.queue.push_back(out_ticket);
                tracing::info!(node_id = %node_config.id, "Executed script");
            }

            // Telemetry
            let elapsed = start.elapsed().as_millis() as u64;
            let _ = event_tx.send(crate::api::events::SystemEvent::NodeTelemetry {
                trace_id,
                node_id: node_config.id,
                node_type: "Script".to_string(),
                execution_ms: elapsed,
                success,
                details: serde_json::json!({
                    "script_len": config.script.len()
                }),
            });
        }
    }
}
