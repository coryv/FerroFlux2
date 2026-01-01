use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::TransformConfig;
use crate::store::BlobStore;
use crate::systems::utils::merge_result;
use bevy_ecs::prelude::*;
use serde_json::json;
use std::time::Instant;

#[tracing::instrument(skip(query, store, event_bus))]
pub fn transform_worker(
    mut query: Query<(&TransformConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            let start = Instant::now();
            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let payload_bytes = match store.claim(&ticket) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            let input_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            let expr = match jmespath::compile(&config.expression) {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, error = %e, "Invalid expression in transform");
                    continue;
                }
            };

            // FIX: Using Rc
            let search_result = expr
                .search(&input_val)
                .unwrap_or_else(|_| std::rc::Rc::new(jmespath::Variable::Null));
            let result_json =
                serde_json::to_value(search_result).unwrap_or(serde_json::Value::Null);
            let result_str = result_json.to_string();

            // FIX: merge_result args
            let final_output = merge_result(&input_val, &result_str, config.result_key.as_ref());

            // Re-parse to validate/store as JSON bytes
            let final_output_bytes = final_output.into_bytes();

            if let Ok(mut new_ticket) = store.check_in(&final_output_bytes) {
                new_ticket.metadata = ticket.metadata.clone();
                outbox.queue.push_back((None, new_ticket));
            }

            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Transform".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({ "message": "Transformation complete" }),
            });
        }
    }
}
