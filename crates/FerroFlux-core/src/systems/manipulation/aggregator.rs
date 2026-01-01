use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::{AggregateConfig, BatchState};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use serde_json::json;
use std::time::{Duration, Instant};

/// System: Aggregator Worker (Fan-In / Batching)
///
/// Buffers incoming items until a condition is met (Size or Time).
/// - **Stateful**: Uses `BatchState` buffer.
/// - Useful for creating batches for `StatsNode` or reducing API calls.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn aggregator_worker(
    mut query: Query<(
        &AggregateConfig,
        &NodeConfig,
        &mut BatchState,
        &mut Inbox,
        &mut Outbox,
    )>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut state, mut inbox, mut outbox) in query.iter_mut() {
        // 1. Ingest
        while let Some(ticket) = inbox.queue.pop_front() {
            if state.items.is_empty() {
                state.last_update = Some(Instant::now());
            }

            if let Ok(bytes) = store.claim(&ticket)
                && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes)
            {
                state.items.push(val);
            }
        }

        // 2. Check Conditions
        let count = state.items.len();
        let elapsed = state
            .last_update
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        let batch_full = count >= config.batch_size && count > 0;
        let timed_out = elapsed >= Duration::from_secs(config.timeout_seconds) && count > 0;

        if batch_full || timed_out {
            let batch_json = serde_json::Value::Array(state.items.clone());

            if let Ok(bytes) = serde_json::to_vec(&batch_json)
                && let Ok(mut ticket) = store.check_in(&bytes)
            {
                ticket
                    .metadata
                    .insert("trace_id".to_string(), uuid::Uuid::new_v4().to_string());
                outbox.queue.push_back((None, ticket));
            }

            // Emit Telemetry
            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Aggregate".to_string(),
                trace_id: "batch_event".to_string(),
                execution_ms: 0,
                success: true,
                details: json!({ "message": format!("Aggregated {} items", count) }),
            });

            // Reset
            state.items.clear();
            state.last_update = None;
        }
    }
}
