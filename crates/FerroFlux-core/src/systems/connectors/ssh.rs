use crate::api::events::SystemEventBus;
use crate::components::connectors::SshConfig;
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::resources::TokioRuntime;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use ferroflux_iam::TenantId;
use serde_json::json;
use std::io::Read;

/// System: SSH Worker
#[tracing::instrument(skip(query, store, secret_store, _event_bus, runtime))]
pub fn ssh_worker(
    mut query: Query<(&SshConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    secret_store: Res<crate::secrets::DatabaseSecretStore>,
    _event_bus: Res<SystemEventBus>,
    runtime: Res<TokioRuntime>,
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
                // Resolving via SecretStore (Blocking for now)
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

                // Auth stub
                if sess.userauth_agent(&user).is_err() {
                    // Fallback to password/key would go here
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
