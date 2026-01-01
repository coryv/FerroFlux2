use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::{
    AuthConfig, HttpConfig, Inbox, NodeConfig, Outbox, PayloadMapper, PinnedOutput, SecretConfig,
};
use ferroflux_iam::TenantId;
use crate::resources::{HttpResultChannel, TokioRuntime, WorkDone};
use crate::secrets::{DatabaseSecretStore, SecretStore};
use crate::store::BlobStore;
use crate::systems::io::auth::resolve_auth_headers;
use crate::systems::io::templating::apply_template;
use crate::systems::utils::merge_result;
use base64::{Engine as _, engine::general_purpose};
use bevy_ecs::prelude::*;
use ipnet::IpNet;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::net::ToSocketAddrs;
use std::time::Instant;
use url::Url;
use uuid::Uuid;

/// System: HTTP I/O Worker
///
/// **Role**: Handles outbound HTTP requests via `reqwest`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
#[tracing::instrument(skip(query, store, work_done, event_bus, channel, secret_store, runtime))]
pub fn http_worker(
    mut query: Query<(
        Entity,
        &HttpConfig,
        &NodeConfig,
        Option<&SecretConfig>,
        Option<&PayloadMapper>,
        Option<&AuthConfig>,
        Option<&PinnedOutput>,
        &mut Inbox,
        &mut Outbox,
    )>,
    store: Res<BlobStore>,
    mut work_done: ResMut<WorkDone>,
    event_bus: Res<SystemEventBus>,
    channel: Res<HttpResultChannel>,
    secret_store: Res<DatabaseSecretStore>,
    runtime: Res<TokioRuntime>,
) {
    let (tx, rx) = (&channel.tx, &channel.rx);
    let event_tx = event_bus.0.clone();

    // 1. Poll Results
    while let Ok((entity, result_str, metadata)) = rx.try_recv() {
        if let Ok((_, _, node_config, _, _, _, _, _, mut outbox)) = query.get_mut(entity) {
            let mut final_metadata = metadata.clone();

            if result_str.starts_with("Error:") {
                final_metadata.insert("status".to_string(), "error".to_string());
                if result_str.contains("Blocked") {
                    final_metadata.insert("status".to_string(), "error_blocked".to_string());
                }
            } else {
                final_metadata.insert("status".to_string(), "ok".to_string());
            }

            let _ = event_tx.send(SystemEvent::AgentActivity {
                node_id: node_config.id,
                activity: "Completed".to_string(),
                content: result_str.clone(),
            });

            if let Ok(ticket) = store.check_in_with_metadata(result_str.as_bytes(), final_metadata)
            {
                tracing::info!(
                    node_id = %node_config.id,
                    ticket_id = %ticket.id,
                    "HTTP Result recorded"
                );
                outbox.queue.push_back((None, ticket));
                work_done.0 = true;
            }
        }
    }

    // 2. Process Requests
    for (
        entity,
        config,
        node_config,
        secret_opt,
        mapper_opt,
        auth_opt,
        pinned_opt,
        mut inbox,
        mut outbox,
    ) in query.iter_mut()
    {
        while let Some(ticket) = inbox.queue.pop_front() {
            if let Some(pinned) = pinned_opt {
                tracing::info!(entity = ?entity, "Node is PINNED. Skipping execution.");
                outbox.queue.push_back((None, pinned.0.clone()));
                work_done.0 = true;
                continue;
            }

            work_done.0 = true;
            let start = Instant::now();

            tracing::debug!(url = %config.url, "Spawning HTTP task");
            let data = match store.claim(&ticket) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            let input_json = serde_json::from_slice::<Value>(&data).ok();

            // Apply Templating
            let data_clone = if let Some(mapper) = mapper_opt {
                if let Some(template) = &mapper.template {
                    if let Some(json) = &input_json {
                        apply_template(template, json).into_bytes()
                    } else {
                        data.to_vec()
                    }
                } else {
                    data.to_vec()
                }
            } else {
                data.to_vec()
            };

            let mut dynamic_headers: Vec<(String, String)> = Vec::new();
            if let Some(mapper) = mapper_opt
                && let Some(json) = &input_json
            {
                for (k, v) in &mapper.headers {
                    let val = apply_template(v, json);
                    dynamic_headers.push((k.clone(), val));
                }
            }

            if let Some(auth_config) = auth_opt {
                let headers = resolve_auth_headers(auth_config);
                dynamic_headers.extend(headers);
            }

            if let Some(secret_config) = secret_opt
                && let Ok(val) = env::var(&secret_config.lookup_key)
            {
                let header_val = secret_config.template.replace("{}", &val);
                dynamic_headers.push((secret_config.header_name.clone(), header_val));
            }

            let mut url_str = config.url.clone();
            let method = config.method.clone();
            let tx_clone = tx.clone();
            let entity_id = entity;
            let input_val_for_merge = input_json.clone().unwrap_or(json!({}));
            let result_key = config.result_key.clone();
            let trace_id_clone = trace_id.clone();
            let event_tx_clone = event_tx.clone();
            let node_id = node_config.id;
            let connection_slug_opt = config.connection_slug.clone();
            let secret_store_clone = secret_store.clone();
            let tenant = node_config
                .tenant_id
                .as_ref()
                .cloned()
                .unwrap_or_else(|| TenantId::from("default_tenant"));

            let _ = event_tx_clone.send(SystemEvent::Log {
                level: "INFO".into(),
                message: format!("HTTP Request to {}", url_str),
                trace_id: trace_id_clone.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            });

            runtime.0.spawn(async move {
                let span = tracing::info_span!("http_request", node_id = %node_id, trace_id = %trace_id_clone);
                let _enter = span.enter();

                if let Some(slug) = connection_slug_opt {
                    match secret_store_clone.resolve_connection(&tenant, &slug).await {
                        Ok(conn_data) => {
                            if let Some(base) = conn_data.get("base_url").and_then(|v| v.as_str()) {
                                let base = base.trim_end_matches('/');
                                let path = url_str.trim_start_matches('/');
                                if path.is_empty() {
                                    url_str = base.to_string();
                                } else {
                                    url_str = format!("{}/{}", base, path);
                                }
                            }

                            if let Some(auth_type) =
                                conn_data.get("auth_type").and_then(|v| v.as_str())
                            {
                                match auth_type {
                                    "Bearer" => {
                                        if let Some(cred) =
                                            conn_data.get("credentials").and_then(|v| v.as_str())
                                        {
                                            dynamic_headers.push((
                                                "Authorization".to_string(),
                                                format!("Bearer {}", cred),
                                            ));
                                        }
                                    }
                                    "Basic" => {
                                        if let Some(cred) =
                                            conn_data.get("credentials").and_then(|v| v.as_str())
                                        {
                                            let encoded = general_purpose::STANDARD.encode(cred);
                                            dynamic_headers.push((
                                                "Authorization".to_string(),
                                                format!("Basic {}", encoded),
                                            ));
                                        }
                                    }
                                    "Custom Scheme" => {
                                        let scheme = conn_data
                                            .get("auth_scheme")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Bearer");

                                        if let Some(cred) =
                                            conn_data.get("credentials").and_then(|v| v.as_str())
                                        {
                                            dynamic_headers.push((
                                                "Authorization".to_string(),
                                                format!("{} {}", scheme, cred),
                                            ));
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if let Some(headers) =
                                conn_data.get("custom_headers").and_then(|v| v.as_object())
                            {
                                for (k, v) in headers {
                                    if let Some(val_str) = v.as_str() {
                                        dynamic_headers.push((k.clone(), val_str.to_string()));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx_clone
                                .send((
                                    entity_id,
                                    format!("Error: Connection Resolution Failed: {}", e),
                                    HashMap::new(),
                                ))
                                .await;
                            return;
                        }
                    }
                }

                let url_for_thread = url_str.clone();
                let result = tokio::task::spawn_blocking(move || {
                    let parsed_url = match Url::parse(&url_for_thread) {
                        Ok(u) => u,
                        Err(e) => return (format!("Error: Invalid URL {}", e), 0),
                    };

                    let host_str = match parsed_url.host_str() {
                        Some(h) => h,
                        None => return ("Error: No Host".to_string(), 0),
                    };

                    let port = parsed_url.port_or_known_default().unwrap_or(80);

                    let socket_addrs = match format!("{}:{}", host_str, port).to_socket_addrs() {
                        Ok(iter) => iter,
                        Err(e) => return (format!("Error: DNS Resolution Failed {}", e), 0),
                    };

                    let allow_internal =
                        env::var("FERROFLUX_ALLOW_INTERNAL_IPS").unwrap_or_default() == "true";

                    if !allow_internal {
                        let blocklist = [
                            "127.0.0.0/8",
                            "10.0.0.0/8",
                            "172.16.0.0/12",
                            "192.168.0.0/16",
                            "169.254.0.0/16",
                        ];

                        for addr in socket_addrs {
                            let ip = addr.ip();
                            for range in &blocklist {
                                if let Ok(net) = range.parse::<IpNet>()
                                    && net.contains(&ip)
                                {
                                    return (format!("Error: Blocked Internal IP {}", ip), 403);
                                }
                            }
                        }
                    }

                    let client = reqwest::blocking::Client::new();
                    let mut request = match method.as_str() {
                        "POST" => client.post(&url_for_thread).body(data_clone),
                        _ => client.get(&url_for_thread),
                    };

                    for (name, val) in dynamic_headers {
                        request = request.header(name, val);
                    }

                    match request.send() {
                        Ok(resp) => {
                            let code = resp.status().as_u16();
                            if resp.status().is_success() {
                                (resp.text().unwrap_or_default(), code)
                            } else {
                                (format!("Error: HTTP {}", resp.status()), code)
                            }
                        }
                        Err(e) => (format!("Error: {}", e), 0),
                    }
                })
                .await;

                if let Ok((result_text, status_code)) = result {
                    let output = merge_result(
                        &input_val_for_merge,
                        &result_text,
                        result_key.as_ref(),
                    );

                    let success = !result_text.starts_with("Error:");
                    let elapsed = start.elapsed().as_millis() as u64;

                    let _ = event_tx_clone.send(SystemEvent::NodeTelemetry {
                        trace_id: trace_id_clone.clone(),
                        node_id,
                        node_type: "Http".to_string(),
                        execution_ms: elapsed,
                        success,
                        details: json!({
                            "url": url_str,
                            "status": status_code
                        }),
                    });

                    let mut out_meta = HashMap::new();
                    out_meta.insert("trace_id".to_string(), trace_id_clone);

                    let _ = tx_clone.send((entity_id, output, out_meta)).await;
                }
            });
        }
    }
}
