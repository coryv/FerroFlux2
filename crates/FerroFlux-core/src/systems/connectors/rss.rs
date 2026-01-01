use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::connectors::{RssConfig, RssState};
use crate::components::core::{NodeConfig, Outbox};
use crate::resources::GlobalHttpClient;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use serde_json::json;

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

        // 1. Validate
        if let Err(e) = ferroflux_security::network::validate_url(&url) {
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
                Err(_) => std::time::SystemTime::now(),
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
