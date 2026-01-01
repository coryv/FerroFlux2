use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::{WindowConfig, WindowOp, WindowState};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use serde_json::json;
use std::time::Instant;

/// System: Window Worker (Rolling Analysis)
///
/// ## Architecture: Stateful Nodes
/// Unlike most nodes which are stateless (f(input) -> output), this node maintains `WindowState`.
/// - `WindowState` contains a generic `VecDeque` buffer.
/// - State persists across ticks in the ECS `Query`.
/// - Allows calculating rolling averages/sums over a stream of individual events.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn window_worker(
    mut query: Query<(
        &WindowConfig,
        &NodeConfig,
        &mut WindowState,
        &mut Inbox,
        &mut Outbox,
    )>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut state, mut inbox, mut outbox) in query.iter_mut() {
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

            let mut json_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            // 1. Extract Value
            let val = if let Some(obj) = json_val.as_object() {
                obj.get(&config.target_field).and_then(|v| v.as_f64())
            } else {
                None
            };

            // 2. Update Window State
            if let Some(v) = val {
                state.buffer.push_back(v);
                if state.buffer.len() > config.window_size {
                    state.buffer.pop_front();
                }
            }

            // 3. Calculate
            let count = state.buffer.len() as f64;
            let result = if count == 0.0 {
                0.0
            } else {
                match config.operation {
                    WindowOp::Mean => {
                        let sum: f64 = state.buffer.iter().sum();
                        sum / count
                    }
                    WindowOp::Sum => state.buffer.iter().sum(),
                    WindowOp::Min => state.buffer.iter().cloned().fold(f64::INFINITY, f64::min),
                    WindowOp::Max => state
                        .buffer
                        .iter()
                        .cloned()
                        .fold(f64::NEG_INFINITY, f64::max),
                    WindowOp::Variance => {
                        let sum: f64 = state.buffer.iter().sum();
                        let mean = sum / count;
                        state
                            .buffer
                            .iter()
                            .map(|v| {
                                let diff = mean - *v;
                                diff * diff
                            })
                            .sum::<f64>()
                            / count
                    }
                }
            };

            // 4. Enrich
            if let Some(obj) = json_val.as_object_mut() {
                obj.insert(config.result_key.clone(), serde_json::json!(result));
            }

            // 5. Output
            if let Ok(bytes) = serde_json::to_vec(&json_val)
                && let Ok(mut new_ticket) = store.check_in(&bytes)
            {
                new_ticket.metadata = ticket.metadata;
                outbox.queue.push_back((None, new_ticket));
            }

            // Telemetry
            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Window".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({
                    "window_size": state.buffer.len(),
                    "result": result
                }),
            });
        }
    }
}
