use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::control::CheckpointConfig;
use crate::components::core::{Inbox, NodeConfig};
use ferroflux_iam::TenantId;
use crate::store::BlobStore;
use crate::store::database::PersistentStore;
use bevy_ecs::prelude::*;
use serde_json::json;
use uuid::Uuid;

#[tracing::instrument(skip(query, store, db, event_bus))]
pub fn checkpoint_worker(
    mut query: Query<(&CheckpointConfig, &NodeConfig, &mut Inbox)>,
    store: Res<BlobStore>,
    db: Res<PersistentStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (_config, node_config, mut inbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let payload_bytes = match store.claim(&ticket) {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, error = %e, "Failed to claim ticket for checkpoint");
                    continue;
                }
            };

            // Generate Token
            let token = Uuid::new_v4().to_string();

            // Spawn tokio task.
            let db_clone = db.clone();
            let event_tx_clone = event_tx.clone();
            let node_id_clone = node_config.id;
            // distinct clones for each usage
            let trace_id_clone_1 = trace_id.clone();
            let trace_id_clone_2 = trace_id.clone();
            let token_clone = token.clone();
            let metadata_clone = ticket.metadata; // Move
            let data_clone = payload_bytes.to_vec(); // Convert Arc to Vec for DB (or fix DB)

            let tenant = node_config
                .tenant_id
                .clone()
                .unwrap_or_else(|| TenantId::from("default_tenant"));

            tokio::spawn(async move {
                let span = tracing::info_span!("checkpoint_save", node_id = %node_id_clone, trace_id = %trace_id_clone_1);
                let _enter = span.enter();

                match db_clone
                    .save_checkpoint(
                        &tenant,
                        &token_clone,
                        node_id_clone,
                        &data_clone,
                        &metadata_clone,
                    )
                    .await
                {
                    Ok(_) => {
                        tracing::info!(token = %token_clone, "Checkpoint saved successfully");
                        // Emit Event 1
                        let _ = event_tx_clone.send(SystemEvent::NodeTelemetry {
                            node_id: node_id_clone,
                            node_type: "Checkpoint".to_string(),
                            trace_id: trace_id_clone_1,
                            execution_ms: 0,
                            success: true,
                            details: json!({
                                "action": "hibernated",
                                "token": token_clone
                            }),
                        });

                        // Emit Event 2
                        let _ = event_tx_clone.send(SystemEvent::CheckpointCreated {
                            token: token_clone,
                            node_id: node_id_clone,
                            trace_id: trace_id_clone_2,
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to save checkpoint to database");
                    }
                }
            });

            // Flow Stops Here. No Outbox Push.
        }
    }
}
