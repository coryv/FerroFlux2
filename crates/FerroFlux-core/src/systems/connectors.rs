use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::connectors::{
    FtpConfig, FtpOperation, FtpProtocol, RssConfig, RssState, SshConfig, XmlConfig,
};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::domain::TenantId;
use crate::resources::GlobalHttpClient;
use crate::store::BlobStore;
use crate::systems::utils::merge_result;
use bevy_ecs::prelude::*;
use serde_json::json;
use std::io::Read;

/// System: RSS Poller
#[tracing::instrument(skip(query, store, _http_client, event_bus, local))]
pub fn rss_worker(
    mut query: Query<(&RssConfig, &NodeConfig, &mut RssState, &mut Outbox)>,
    store: Res<BlobStore>,
    _http_client: Res<GlobalHttpClient>,
    event_bus: Res<SystemEventBus>,
    mut local: Local<Option<std::time::Instant>>,
) {
    let event_tx = event_bus.0.clone();

    // Simple throttle (10s global)
    if let Some(last) = *local
        && last.elapsed() < std::time::Duration::from_secs(10)
    {
        return;
    }
    *local = Some(std::time::Instant::now());

    for (config, node_config, mut state, mut outbox) in query.iter_mut() {
        let url = config.url.clone();
        let node_id = node_config.id;
        let event_tx_clone = event_tx.clone();

        // BLOCKING IO (Acceptable for MVP with throttle)
        // 1. Validate
        if let Err(e) = crate::security::network::validate_url(&url) {
            let _ = event_tx_clone.send(SystemEvent::NodeTelemetry {
                node_id,
                node_type: "RSS".into(),
                trace_id: "system".into(),
                execution_ms: 0,
                success: false,
                details: json!({ "error": format!("Security Validation Failed: {}", e) }),
            });
            continue;
        }

        // 2. Fetch
        let resp = match reqwest::blocking::get(&url) {
            Ok(r) => r,
            Err(e) => {
                let _ = event_tx_clone.send(SystemEvent::NodeTelemetry {
                    node_id,
                    node_type: "RSS".into(),
                    trace_id: "system".into(),
                    execution_ms: 0,
                    success: false,
                    details: json!({ "error": e.to_string() }),
                });
                continue;
            }
        };

        // 2. Parse
        let content = match resp.bytes() {
            Ok(b) => b,
            Err(_) => continue,
        };

        let channel = match rss::Channel::read_from(&content[..]) {
            Ok(c) => c,
            Err(e) => {
                let _ = event_tx_clone.send(SystemEvent::NodeTelemetry {
                    node_id,
                    node_type: "RSS".into(),
                    trace_id: "system".into(),
                    execution_ms: 0,
                    success: false,
                    details: json!({ "error": format!("RSS Parse Error: {}", e) }),
                });
                continue;
            }
        };

        // 3. Filter & Emit
        let mut max_date: Option<std::time::SystemTime> = state.last_pub_date;
        let mut emit_count = 0;

        for item in channel.items() {
            // Parse date
            let pub_date_str = item.pub_date().unwrap_or("");
            let pub_date = match chrono::DateTime::parse_from_rfc2822(pub_date_str) {
                Ok(d) => std::time::SystemTime::from(d),
                Err(_) => std::time::SystemTime::now(), // Fallback
            };

            // If newer than last seen
            let is_new = match state.last_pub_date {
                Some(last) => pub_date > last,
                None => true,
            };

            if is_new {
                // Emit Ticket
                let payload = json!({
                    "title": item.title(),
                    "link": item.link(),
                    "description": item.description(),
                    "pubDate": pub_date_str,
                    "guid": item.guid().map(|g| g.value()).unwrap_or(""),
                });

                // Update max seen
                if max_date.map(|m| pub_date > m).unwrap_or(true) {
                    max_date = Some(pub_date);
                }

                // Store & Push
                if let Ok(bytes) = serde_json::to_vec(&payload)
                    && let Ok(mut ticket) = store.check_in(&bytes)
                {
                    ticket
                        .metadata
                        .insert("trace_id".into(), uuid::Uuid::new_v4().to_string());
                    outbox.queue.push_back((None, ticket));
                    emit_count += 1;
                }
            }
        }
        // Update State
        state.last_pub_date = max_date;

        if emit_count > 0 {
            let _ = event_tx_clone.send(SystemEvent::NodeTelemetry {
                node_id,
                node_type: "RSS".into(),
                trace_id: "system".into(),
                execution_ms: 0,
                success: true,
                details: json!({ "message": "Polled RSS", "new_items": emit_count }),
            });
        }
    }
}

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
                // Try whole payload as string? Or maybe payload IS the xml bytes?
                // Usually payload is JSON. If it's pure XML bytes, from_slice above failed
                // and input_val is Null.
                // If from_slice failed, check if bytes are utf8 xml
                std::str::from_utf8(&payload_bytes)
                    .ok()
                    .map(|s| s.to_string())
            };

            if let Some(xml) = xml_str {
                // Parse XML -> JSON
                // quick-xml generic deserialize is tricky.
                // We'll use a simple approach: if it fails, error.

                // Using a crate like `serde-xml-rs` is often easier for this,
                // but quick-xml + serde feature is supported.
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
                            trace_id,
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
                            trace_id,
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

/// System: SSH Worker
#[tracing::instrument(skip(query, store, secret_store, _event_bus, runtime))]
pub fn ssh_worker(
    mut query: Query<(&SshConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    secret_store: Res<crate::secrets::DatabaseSecretStore>,
    _event_bus: Res<SystemEventBus>,
    runtime: Res<crate::resources::TokioRuntime>,
) {
    use crate::secrets::SecretStore;

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            let tenant = node_config
                .tenant_id
                .as_ref()
                .cloned()
                .unwrap_or_else(|| TenantId::from("default_tenant"));

            // Resolve Credentials
            let (user, _key_secret) = if let Some(slug) = &config.connection_slug {
                // Resolving via SecretStore (Blocking for now, consistent with blocking IO of this system)
                let rt = runtime.clone();
                let ss = secret_store.clone();
                match tokio::task::block_in_place(move || {
                    rt.0.block_on(ss.resolve_connection(&tenant, slug))
                }) {
                    Ok(json) => {
                        let u = json
                            .get("username")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&config.user_secret)
                            .to_string();
                        let p = json
                            .get("password")
                            .or(json.get("private_key"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(&config.key_secret)
                            .to_string();
                        (u, p)
                    }
                    Err(_) => (config.user_secret.clone(), config.key_secret.clone()),
                }
            } else {
                (config.user_secret.clone(), config.key_secret.clone())
            };

            use std::net::TcpStream;

            if let Err(e) = crate::security::network::validate_host_port(&config.host, config.port)
            {
                tracing::error!("SSH Security Validation Failed: {}", e);
                continue;
            }

            let tcp = match TcpStream::connect(format!("{}:{}", config.host, config.port)) {
                Ok(t) => t,
                Err(_) => continue,
            };

            if let Ok(mut sess) = ssh2::Session::new() {
                sess.set_tcp_stream(tcp);
                sess.handshake().unwrap();

                // Auth
                if sess.userauth_agent(&user).is_err() {
                    // Fallback to password/key if agent fails or not used
                    // This is a stub for where actual auth logic with `key_secret` would go
                }

                if sess.authenticated()
                    && let Ok(mut channel) = sess.channel_session()
                {
                    channel.exec(&config.command).unwrap();
                    let mut s = String::new();
                    channel.read_to_string(&mut s).unwrap();

                    let payload = json!({
                        "stdout": s,
                        "exit_code": channel.exit_status().unwrap_or(0)
                    });

                    if let Ok(bytes) = serde_json::to_vec(&payload)
                        && let Ok(mut t) = store.check_in(&bytes)
                    {
                        t.metadata = ticket.metadata.clone();
                        outbox.queue.push_back((None, t));
                    }
                }
            }
        }
    }
}

/// System: FTP Worker
#[tracing::instrument(skip(query, store, secret_store, _event_bus, runtime))]
pub fn ftp_worker(
    mut query: Query<(&FtpConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    secret_store: Res<crate::secrets::DatabaseSecretStore>,
    _event_bus: Res<SystemEventBus>,
    runtime: Res<crate::resources::TokioRuntime>,
) {
    use crate::secrets::SecretStore;

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        // Trigger on incoming ticket
        while let Some(ticket) = inbox.queue.pop_front() {
            let tenant = node_config
                .tenant_id
                .as_ref()
                .cloned()
                .unwrap_or_else(|| TenantId::from("default_tenant"));

            // Resolve Credentials
            let (user, pass) = if let Some(slug) = &config.connection_slug {
                let rt = runtime.clone();
                let ss = secret_store.clone();
                match tokio::task::block_in_place(move || {
                    rt.0.block_on(ss.resolve_connection(&tenant, slug))
                }) {
                    Ok(json) => {
                        let u = json
                            .get("username")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&config.user_secret)
                            .to_string();
                        let p = json
                            .get("password")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&config.pass_secret)
                            .to_string();
                        (u, p)
                    }
                    Err(_) => (config.user_secret.clone(), config.pass_secret.clone()),
                }
            } else {
                (config.user_secret.clone(), config.pass_secret.clone())
            };

            // Use suppaftp for generic FTP
            if config.protocol == FtpProtocol::Ftp {
                use suppaftp::FtpStream;
                let addr = format!("{}:{}", config.host, config.port);

                if let Err(e) =
                    crate::security::network::validate_host_port(&config.host, config.port)
                {
                    tracing::error!("FTP Security Validation Failed: {}", e);
                    continue;
                }

                if let Ok(mut ftp) = FtpStream::connect(addr) {
                    let _ = ftp.login(&user, &pass);

                    match config.operation {
                        FtpOperation::List => {
                            if let Ok(files) = ftp.list(Some(&config.path)) {
                                let payload = json!({ "files": files });
                                if let Ok(bytes) = serde_json::to_vec(&payload)
                                    && let Ok(mut t) = store.check_in(&bytes)
                                {
                                    t.metadata = ticket.metadata.clone();
                                    outbox.queue.push_back((None, t));
                                }
                            }
                        }
                        FtpOperation::Get => {
                            // Download logic stub
                        }
                        FtpOperation::Put => {
                            // Upload logic stub
                        }
                    }
                }
            }
            // SFTP would use ssh2 (see ssh_worker)
        }
    }
}
