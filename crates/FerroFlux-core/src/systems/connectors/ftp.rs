use crate::api::events::SystemEventBus;
use crate::components::connectors::{FtpConfig, FtpOperation, FtpProtocol};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::resources::TokioRuntime;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use ferroflux_iam::TenantId;
use serde_json::json;

/// System: FTP Worker
#[tracing::instrument(skip(query, store, secret_store, _event_bus, runtime))]
pub fn ftp_worker(
    mut query: Query<(&FtpConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
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
                        FtpOperation::Get => {}
                        FtpOperation::Put => {}
                    }
                }
            }
        }
    }
}
