use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::SplitConfig;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use serde_json::json;
use std::time::Instant;

/// System: Splitter Worker (Fan-Out)
///
/// Takes a single Ticket containing an Array and produces N Tickets.
/// - Uses JMESPath to locate the array within the payload.
/// - **Zero-Copy Optimization**: While we deserialize the JSON to split it,
///   outgoing tickets get their own fresh entries in BlobStore to ensure independent ownership.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn splitter_worker(
    mut query: Query<(&SplitConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
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
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, error = %e, "Failed to claim ticket for splitter");
                    continue;
                }
            };

            let input_val: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
                Ok(v) => v,
                Err(_) => serde_json::Value::Null,
            };

            // Determine array to split
            let path = config.path.as_deref().unwrap_or("@"); // @ is current node (root) in JMESPath
            let expr = match jmespath::compile(path) {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, path = %path, error = %e, "Invalid JMESPath in splitter");
                    continue;
                }
            };

            // FIX: Using std::rc::Rc because jmespath::Expression::search returns Rc<Variable>
            let search_result = expr
                .search(&input_val)
                .unwrap_or_else(|_| std::rc::Rc::new(jmespath::Variable::Null));

            let array_val = serde_json::to_value(search_result).unwrap_or(serde_json::Value::Null);

            let array = if array_val.is_array() {
                array_val.as_array().unwrap().clone()
            } else if array_val.is_null() {
                vec![]
            } else {
                vec![array_val.clone()]
            };

            let count = array.len();

            for item in array {
                if let Ok(bytes) = serde_json::to_vec(&item)
                    && let Ok(new_ticket) = store.check_in(&bytes)
                {
                    let mut final_ticket = new_ticket;
                    final_ticket.metadata = ticket.metadata.clone();
                    outbox.queue.push_back((None, final_ticket));
                }
            }

            // Emit Telemetry
            // FIX: Added node_type, converted details to Value
            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Split".to_string(),
                trace_id: trace_id.clone(),
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({ "message": format!("Split into {} items", count) }),
            });
        }
    }
}
