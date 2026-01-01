use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::connectors::XmlConfig;
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::store::BlobStore;
use crate::systems::utils::merge_result;
use bevy_ecs::prelude::*;
use serde_json::json;

/// System: XML Transformer
#[tracing::instrument(skip(query, store, event_bus))]
pub fn xml_worker(
    mut query: Query<(&XmlConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            let start = std::time::Instant::now();
            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or("unknown".into());

            let payload_bytes = match store.claim(&ticket) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let input_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            // Extract XML String
            let xml_str = if let Some(field) = &config.target_field {
                input_val
                    .get(field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                std::str::from_utf8(&payload_bytes)
                    .ok()
                    .map(|s| s.to_string())
            };

            if let Some(xml) = xml_str {
                // Parse XML -> JSON
                match quick_xml::de::from_str::<serde_json::Value>(&xml) {
                    Ok(json_val) => {
                        let result_str = json_val.to_string();
                        let final_json =
                            merge_result(&input_val, &result_str, config.result_key.as_ref());

                        if let Ok(bytes) = serde_json::to_vec(
                            &serde_json::from_str::<serde_json::Value>(&final_json).unwrap(),
                        ) && let Ok(mut new_ticket) = store.check_in(&bytes)
                        {
                            new_ticket.metadata = ticket.metadata.clone();
                            outbox.queue.push_back((None, new_ticket));
                        }

                        let _ = event_tx.send(SystemEvent::NodeTelemetry {
                            node_id: node_config.id,
                            node_type: "XML".into(),
                            trace_id: trace_id.clone(),
                            execution_ms: start.elapsed().as_millis() as u64,
                            success: true,
                            details: json!({"message": "XML Parsed"}),
                        });
                    }
                    Err(e) => {
                        tracing::error!(node_id = %node_config.id, error = %e, "XML parse error");
                        let _ = event_tx.send(SystemEvent::NodeTelemetry {
                            node_id: node_config.id,
                            node_type: "XML".into(),
                            trace_id: trace_id.clone(),
                            execution_ms: start.elapsed().as_millis() as u64,
                            success: false,
                            details: json!({"error": e.to_string()}),
                        });
                    }
                }
            }
        }
    }
}
