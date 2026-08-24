use crate::components::SseTriggerConfig;
use bevy_ecs::prelude::*;
use ferroflux_types::resources::{SseConnectionHandle, SseTriggerRegistry};
use ferroflux_types::{BlobStore, NodeConfig, TokioRuntime, TriggerEvent, TriggerSender};
use futures::StreamExt;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub fn sse_trigger_system(
    mut registry: ResMut<SseTriggerRegistry>,
    tokio_runtime: Res<TokioRuntime>,
    trigger_sender: Res<TriggerSender>,
    blob_store: Res<BlobStore>,
    query: Query<(Entity, &NodeConfig, &SseTriggerConfig)>,
) {
    let mut active_identities = HashMap::new();

    for (entity, node_config, sse_config) in query.iter() {
        if node_config.node_type != "SseTrigger" {
            continue;
        }

        let identity = (node_config.workflow_id.clone(), node_config.id);
        active_identities.insert(identity.clone(), entity);

        // Calculate config hash
        let mut hasher = DefaultHasher::new();
        sse_config.url.hash(&mut hasher);
        // Sort headers to ensure deterministic hash
        let mut sorted_headers: Vec<_> = sse_config.headers.iter().collect();
        sorted_headers.sort_by_key(|(k, _)| *k);
        for (k, v) in sorted_headers {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        let current_hash = hasher.finish();

        let mut needs_spawn = false;

        if let Some(handle) = registry.connections.get(&identity) {
            if handle.config_hash != current_hash {
                tracing::info!(node_id = %node_config.id, "SSE Config changed, restarting connection");
                handle.abort_handle.abort();
                needs_spawn = true;
            }
        } else {
            needs_spawn = true;
        }

        if needs_spawn {
            let abort_handle = spawn_sse_connection(
                identity.clone(),
                sse_config.clone(),
                tokio_runtime.0.clone(),
                trigger_sender.0.clone(),
                blob_store.clone(),
            );

            registry.connections.insert(
                identity,
                SseConnectionHandle {
                    abort_handle,
                    config_hash: current_hash,
                },
            );
        }
    }

    // Cleanup stale connections
    let mut to_remove = Vec::new();
    for identity in registry.connections.keys() {
        if !active_identities.contains_key(identity) {
            to_remove.push(identity.clone());
        }
    }

    for identity in to_remove {
        if let Some(handle) = registry.connections.remove(&identity) {
            tracing::info!(workflow_id = %identity.0, node_id = %identity.1, "SSE Node removed, aborting connection");
            handle.abort_handle.abort();
        }
    }
}

fn spawn_sse_connection(
    identity: (String, Uuid),
    config: SseTriggerConfig,
    runtime: tokio::runtime::Handle,
    sender: async_channel::Sender<TriggerEvent>,
    blob_store: BlobStore,
) -> tokio::task::AbortHandle {
    let (_workflow_id, node_id) = identity;

    let join_handle = runtime.spawn(async move {
        let mut attempts = 0;
        let max_attempts = config.max_reconnect_attempts;

        loop {
            tracing::info!(node_id = %node_id, url = %config.url, "Connecting to SSE stream");

            // Prevent SSRF by validating the URL against private/internal IP ranges before making the request
            if let Err(e) = ferroflux_security::network::validate_url(&config.url) {
                tracing::error!(node_id = %node_id, error = %e, "SSE Security Validation Failed");
                break;
            }

            let mut request = reqwest::Client::new().get(&config.url);
            for (k, v) in &config.headers {
                request = request.header(k, v);
            }
            request = request.header("Accept", "text/event-stream");

            let res = request.send().await;

            match res {
                Ok(response) => {
                    tracing::info!(node_id = %node_id, status = %response.status(), "SSE Connection established");
                    attempts = 0; // Reset on success
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();

                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(bytes) => {
                                if let Ok(s) = std::str::from_utf8(&bytes) {
                                    buffer.push_str(s);
                                    process_sse_buffer(&mut buffer, &node_id, &sender, &blob_store)
                                        .await;
                                }
                            }
                            Err(e) => {
                                tracing::error!(node_id = %node_id, error = ?e, "SSE Stream error");
                                break;
                            }
                        }
                    }
                    tracing::info!(node_id = %node_id, "SSE stream ended");
                }
                Err(e) => {
                    tracing::error!(node_id = %node_id, error = ?e, "Failed to connect to SSE stream");
                }
            }

            attempts += 1;
            if max_attempts > 0 && attempts >= max_attempts {
                tracing::error!(node_id = %node_id, "Max SSE reconnection attempts reached");
                break;
            }

            tracing::info!(node_id = %node_id, delay_ms = config.reconnect_delay_ms, "Waiting before reconnect");
            tokio::time::sleep(std::time::Duration::from_millis(config.reconnect_delay_ms)).await;
        }
    });

    join_handle.abort_handle()
}

async fn process_sse_buffer(
    buffer: &mut String,
    node_id: &Uuid,
    sender: &async_channel::Sender<TriggerEvent>,
    blob_store: &BlobStore,
) {
    while let Some(pos) = buffer.find("\n\n") {
        let chunk = buffer.drain(..pos + 2).collect::<String>();
        let mut event_type = "message".to_string();
        let mut data = String::new();
        let mut id = None;

        for line in chunk.lines() {
            if let Some(val) = line.strip_prefix("data: ") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(val);
            } else if let Some(val) = line.strip_prefix("event: ") {
                event_type = val.to_string();
            } else if let Some(val) = line.strip_prefix("id: ") {
                id = Some(val.to_string());
            }
        }

        if !data.is_empty() {
            let payload = serde_json::json!({
                "event": event_type,
                "data": data,
                "id": id,
            });

            let mut metadata = HashMap::new();
            metadata.insert("trigger".to_string(), "sse".to_string());
            metadata.insert("node_id".to_string(), node_id.to_string());

            if let Ok(payload_bytes) = serde_json::to_vec(&payload)
                && let Ok(ticket) = blob_store.check_in_with_metadata(&payload_bytes, metadata)
            {
                let _ = sender.try_send(TriggerEvent {
                    trigger_id: *node_id,
                    payload: ticket,
                });
            }
        }
    }
}
