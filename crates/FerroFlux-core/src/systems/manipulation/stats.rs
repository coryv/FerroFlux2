use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::StatsConfig;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use serde_json::json;
use std::time::Instant;

/// System: Statistics Worker (Aggregation Analysis)
///
/// ## Architecture: Vector-First Processing
/// Instead of updating statistics incrementally per item (which is complex for Z-Score/StdDev),
/// this system processes entire batches (Tickets containing Arrays).
/// 1. **Pass 1**: Iterate array to calculate Mean and Variance.
/// 2. **Pass 2**: Iterate array again to calculate Z-Score and flag Outliers.
/// This ensures global context for the batch is available.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn stats_worker(
    mut query: Query<(&StatsConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
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

            // Ensure input is Array
            let array = match input_val.as_array() {
                Some(arr) => arr,
                None => {
                    tracing::warn!(node_id = %node_config.id, "Input to stats is not an array");
                    continue; // Skip or Error
                }
            };

            // Pass 1: Collect Values & Calculate Stats
            let mut values: Vec<f64> = Vec::new();
            for item in array {
                if let Some(val) = item.get(&config.target_field).and_then(|v| v.as_f64()) {
                    values.push(val);
                }
            }

            let count = values.len() as f64;
            if count == 0.0 {
                // No values to analyze, pass through or error?
                // Let's pass through unmodified for safety, maybe logging warning.
                if let Ok(new_ticket) = store.check_in(&payload_bytes) {
                    let mut final_ticket = new_ticket;
                    final_ticket.metadata = ticket.metadata;
                    outbox.queue.push_back((None, final_ticket));
                }
                continue;
            }

            let sum: f64 = values.iter().sum();
            let mean = sum / count;

            let variance: f64 = values
                .iter()
                .map(|v| {
                    let diff = mean - *v;
                    diff * diff
                })
                .sum::<f64>()
                / count;
            let std_dev = variance.sqrt();

            // Pass 2: Enrich
            let mut enriched_array = array.clone();
            let mut outlier_count = 0;

            // Correct Pass 2: Iterate array, if field exists, calc Z-Score
            for item in enriched_array.iter_mut() {
                if let Some(obj) = item.as_object_mut()
                    && let Some(val) = obj.get(&config.target_field).and_then(|v| v.as_f64())
                {
                    let z_score = if std_dev == 0.0 {
                        0.0
                    } else {
                        (val - mean) / std_dev
                    };
                    let is_outlier = config.detect_outliers && z_score.abs() > config.threshold;

                    if is_outlier {
                        outlier_count += 1;
                    }

                    let stats_obj = json!({
                        "mean": mean,
                        "std_dev": std_dev,
                        "z_score": z_score,
                        "is_outlier": is_outlier
                    });

                    obj.insert(config.enrichment_key.clone(), stats_obj);
                }
            }

            let final_json = serde_json::Value::Array(enriched_array);
            let final_bytes = serde_json::to_vec(&final_json).unwrap(); // Should safeguard

            if let Ok(mut new_ticket) = store.check_in(&final_bytes) {
                new_ticket.metadata = ticket.metadata.clone();
                outbox.queue.push_back((None, new_ticket));
            }

            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Stats".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({
                    "count": count,
                    "mean": mean,
                    "std_dev": std_dev,
                    "outliers": outlier_count
                }),
            });
        }
    }
}
