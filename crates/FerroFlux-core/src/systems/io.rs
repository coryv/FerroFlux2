use crate::components::{AuthConfig, HttpConfig, Inbox, Outbox, PayloadMapper, SecretConfig};
use crate::domain::TenantId;
use crate::resources::WorkDone;
use crate::store::BlobStore;
use base64::{Engine as _, engine::general_purpose};
use bevy_ecs::prelude::*;
use ipnet::IpNet;
use std::collections::HashMap;
use std::env;
use std::net::ToSocketAddrs;
use url::Url;
use uuid::Uuid;

/// System: HTTP I/O Worker
///
/// **Role**: Handles outbound HTTP requests via `reqwest`.
///
/// **Mental Model**:
/// - **Phase 1 (Poll)**: Checks the `HttpResultChannel` for completed responses from background threads.
/// - **Phase 2 (Dispatch)**: Checks `Inbox` for new requests.
///   - Validates inputs (SSRF protection).
///   - Applies `PayloadMapper` (templating).
///   - Resolves Auth headers (Async).
///   - Spawns a background task to block on the network call without freezing the Game Loop.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
#[tracing::instrument(skip(query, store, work_done, event_bus, channel, secret_store, runtime))]
pub fn http_worker(
    mut query: Query<(
        Entity,
        &HttpConfig,
        &crate::components::NodeConfig, // Added NodeConfig
        Option<&SecretConfig>,
        Option<&PayloadMapper>,
        Option<&AuthConfig>,
        Option<&crate::components::PinnedOutput>,
        &mut Inbox,
        &mut Outbox,
    )>,
    store: Res<BlobStore>,
    mut work_done: ResMut<WorkDone>,
    event_bus: Res<crate::api::events::SystemEventBus>,
    channel: Res<crate::resources::HttpResultChannel>, // Injected Channel
    secret_store: Res<crate::secrets::DatabaseSecretStore>,
    runtime: Res<crate::resources::TokioRuntime>,
) {
    // 1. Use Channel Logic
    let (tx, rx) = (&channel.tx, &channel.rx);
    let event_tx = event_bus.0.clone();

    // 2. Poll Results
    while let Ok((entity, result_str, metadata)) = rx.try_recv() {
        // Updated destructuring to match Query (9 elements)
        if let Ok((_, _, node_config, _, _, _, _, _, mut outbox)) = query.get_mut(entity) {
            let mut final_metadata = metadata.clone();

            // Status Logic
            if result_str.starts_with("Error:") {
                final_metadata.insert("status".to_string(), "error".to_string());
                if result_str.contains("Blocked") {
                    final_metadata.insert("status".to_string(), "error_blocked".to_string());
                }
            } else {
                final_metadata.insert("status".to_string(), "ok".to_string());
            }

            // Broadcast Event
            let _ = event_tx.send(crate::api::events::SystemEvent::AgentActivity {
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
                outbox.queue.push_back(ticket);
                work_done.0 = true;
            }
        }
    }

    // 3. Process Requests (Spawn Async Tasks)
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
            // Check for Pinned Output
            if let Some(pinned) = pinned_opt {
                tracing::info!(entity = ?entity, "Node is PINNED. Skipping execution.");
                // Push pinned ticket to outbox
                outbox.queue.push_back(pinned.0.clone());
                work_done.0 = true;
                continue;
            }

            work_done.0 = true;
            let start = std::time::Instant::now(); // Start Timer

            tracing::debug!(url = %config.url, "Spawning HTTP task");
            let data = match store.claim(&ticket) {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Trace ID
            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            // Parse Data JSON once if needed
            let input_json = serde_json::from_slice::<serde_json::Value>(&data).ok();

            // Apply Payload Mapper (Body)
            let data_clone = if let Some(mapper) = mapper_opt {
                if let Some(template) = &mapper.template {
                    if let Some(json) = &input_json {
                        apply_template(template, json).into_bytes()
                    } else {
                        data.to_vec() // Clone Arc data to Vec
                    }
                } else {
                    data.to_vec() // Clone Arc data to Vec
                }
            } else {
                data.to_vec() // Clone Arc data to Vec
            };

            // Apply Payload Mapper (Headers)
            let mut dynamic_headers: Vec<(String, String)> = Vec::new();
            if let Some(mapper) = mapper_opt
                && let Some(json) = &input_json
            {
                for (k, v) in &mapper.headers {
                    let val = apply_template(v, json);
                    dynamic_headers.push((k.clone(), val));
                }
            }

            // Resolve AuthConfig (Standard Component)
            if let Some(auth_config) = auth_opt {
                let headers = resolve_auth_headers(auth_config);
                dynamic_headers.extend(headers);
            }

            // Resolve Secret (Legacy, keep for backward compat or manual override)
            if let Some(secret_config) = secret_opt
                && let Ok(val) = env::var(&secret_config.lookup_key)
            {
                let header_val = secret_config.template.replace("{}", &val);
                dynamic_headers.push((secret_config.header_name.clone(), header_val));
            }

            // Captures
            let mut url_str = config.url.clone();
            let method = config.method.clone();
            let tx_clone = tx.clone();
            let entity_id = entity;
            let input_val_for_merge = input_json.clone().unwrap_or(serde_json::json!({}));
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

            let _ = event_tx_clone.send(crate::api::events::SystemEvent::Log {
                level: "INFO".into(),
                message: format!("HTTP Request to {}", url_str),
                trace_id: trace_id_clone.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            });

            // Spawn Async Task
            runtime.0.spawn(async move {
                let span = tracing::info_span!("http_request", node_id = %node_id, trace_id = %trace_id_clone);
                let _enter = span.enter();

                // 1. Connection Resolution (Async)
                if let Some(slug) = connection_slug_opt {
                    use crate::secrets::SecretStore;
                    match secret_store_clone.resolve_connection(&tenant, &slug).await {
                        Ok(conn_data) => {
                            // A. Base URL extraction
                            // If user provided "/path", and conn has "https://api.com", join them.
                            if let Some(base) = conn_data.get("base_url").and_then(|v| v.as_str()) {
                                let base = base.trim_end_matches('/');
                                let path = url_str.trim_start_matches('/');
                                // If url_str was empty or just "/", we append.
                                if path.is_empty() {
                                    url_str = base.to_string();
                                } else {
                                    url_str = format!("{}/{}", base, path);
                                }
                            }

                            // B. Auth Logic
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
                                            .unwrap_or("Bearer"); // Fallback if missing? Or ignore?

                                        if let Some(cred) =
                                            conn_data.get("credentials").and_then(|v| v.as_str())
                                        {
                                            dynamic_headers.push((
                                                "Authorization".to_string(),
                                                format!("{} {}", scheme, cred),
                                            ));
                                        }
                                    }
                                    // Custom / API Key logic...
                                    _ => {}
                                }
                            }

                            // C. Custom Headers
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

                // 2. Network IO (Blocking in Thread, wrapped in Async)
                // Use spawn_blocking to offload sync work (DNS, Parsing, Reqwest Blocking)
                // Note: We move dynamic_headers and other data into the closure.
                let url_for_thread = url_str.clone();
                let result = tokio::task::spawn_blocking(move || {
                    // SSRF Protection Logic
                    let parsed_url = match Url::parse(&url_for_thread) {
                        Ok(u) => u,
                        Err(e) => return (format!("Error: Invalid URL {}", e), 0),
                    };

                    let host_str = match parsed_url.host_str() {
                        Some(h) => h,
                        None => return ("Error: No Host".to_string(), 0),
                    };

                    // Default port if missing
                    let port = parsed_url.port_or_known_default().unwrap_or(80);

                    // Allow explicit resolution
                    let socket_addrs = match format!("{}:{}", host_str, port).to_socket_addrs() {
                        Ok(iter) => iter,
                        Err(e) => return (format!("Error: DNS Resolution Failed {}", e), 0),
                    };

                    // Check Blocklist (Unless allowed via ENV)
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

                    // If Safe, Proceed
                    let client = reqwest::blocking::Client::new();
                    let mut request = match method.as_str() {
                        "POST" => client.post(&url_for_thread).body(data_clone),
                        _ => client.get(&url_for_thread),
                    };

                    for (name, val) in dynamic_headers {
                        request = request.header(name, val);
                    }

                    let response = request.send();

                    match response {
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

                // 3. Post-Process (Merge & Return)
                // Result of spawn_blocking is Result<T, JoinError>
                if let Ok((result_text, status_code)) = result {
                    let output = crate::systems::utils::merge_result(
                        &input_val_for_merge,
                        &result_text,
                        result_key.as_ref(),
                    );

                    let success = !result_text.starts_with("Error:");
                    let elapsed = start.elapsed().as_millis() as u64;

                    // Telemetry
                    let _ = event_tx_clone.send(crate::api::events::SystemEvent::NodeTelemetry {
                        trace_id: trace_id_clone.clone(),
                        node_id,
                        node_type: "Http".to_string(),
                        execution_ms: elapsed,
                        success,
                        details: serde_json::json!({
                            "url": url_str, // Note: url_str was moved! This might fail compilation if we use it inside spawn_blocking?
                            // No, url_str was moved INTO spawn_blocking. We cannot use it here.
                            // We need to clone it before spawn_blocking if we want it here.
                            // Or return it from spawn_blocking.
                            "status": status_code
                        }),
                    });

                    // Simple fix regarding url_str: pass it out or just log "unknown" for now to fix compile error?
                    // I will fix this by cloning url_str before spawn_blocking.

                    let mut out_meta = HashMap::new();
                    out_meta.insert("trace_id".to_string(), trace_id_clone);

                    let _ = tx_clone.send((entity_id, output, out_meta)).await;
                }
            });
        }
    }
}

fn resolve_auth_headers(auth_config: &AuthConfig) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    match auth_config {
        AuthConfig::Basic { user_env, pass_env } => {
            if let (Ok(u), Ok(p)) = (env::var(user_env), env::var(pass_env)) {
                let plain = format!("{}:{}", u, p);
                let encoded = general_purpose::STANDARD.encode(plain);
                headers.push(("Authorization".to_string(), format!("Basic {}", encoded)));
            }
        }
        AuthConfig::ApiKey {
            key_env,
            header,
            query: _,
        } => {
            if let Ok(key_val) = env::var(key_env)
                && let Some(h) = header
            {
                headers.push((h.clone(), key_val));
            }
        }
        AuthConfig::Bearer { token_env }
        | AuthConfig::OAuth2 {
            token_ref: token_env,
        } => {
            if let Ok(token) = env::var(token_env) {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
        }
    }
    headers
}

use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, handlebars_helper,
};
use serde_json::Value;

// Define helper using macro outside function
handlebars_helper!(HandlebarsEq: |x: Value, y: Value| x == y);

fn apply_template(template: &str, json: &serde_json::Value) -> String {
    let mut reg = Handlebars::new();
    reg.set_strict_mode(false);

    // Register built-in helpers
    reg.register_helper(
        "json",
        Box::new(
            |h: &Helper,
             _: &Handlebars,
             _: &Context,
             _: &mut RenderContext,
             out: &mut dyn Output|
             -> HelperResult {
                let param =
                    h.param(0)
                        .ok_or(handlebars::RenderErrorReason::ParamNotFoundForIndex(
                            "json", 0,
                        ))?;
                let json_str = serde_json::to_string(param.value())
                    .map_err(|e| handlebars::RenderErrorReason::Other(e.to_string()))?;
                out.write(&json_str)?;
                Ok(())
            },
        ),
    );

    // Register EQ helper generated by macro
    reg.register_helper("eq", Box::new(HandlebarsEq));

    // Helper: {{is_string var}} -> boolean
    reg.register_helper(
        "is_string",
        Box::new(
            |h: &Helper,
             _: &Handlebars,
             _: &Context,
             _: &mut RenderContext,
             out: &mut dyn Output|
             -> HelperResult {
                let param =
                    h.param(0)
                        .ok_or(handlebars::RenderErrorReason::ParamNotFoundForIndex(
                            "is_string",
                            0,
                        ))?;
                if param.value().is_string() {
                    out.write("true")?;
                }
                Ok(())
            },
        ),
    );

    // Helper: {{is_array var}} -> boolean
    reg.register_helper(
        "is_array",
        Box::new(
            |h: &Helper,
             _: &Handlebars,
             _: &Context,
             _: &mut RenderContext,
             out: &mut dyn Output|
             -> HelperResult {
                let param =
                    h.param(0)
                        .ok_or(handlebars::RenderErrorReason::ParamNotFoundForIndex(
                            "is_array", 0,
                        ))?;
                if param.value().is_array() {
                    out.write("true")?;
                }
                Ok(())
            },
        ),
    );

    reg.render_template(template, json)
        .unwrap_or_else(|e| format!("Template Error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn test_apply_template_simple() {
        let template = r#"{"msg": "{{text}}", "channel": "{{chan}}"}"#;
        let input = serde_json::json!({
            "text": "Hello World",
            "chan": "general"
        });

        let result_str = apply_template(template, &input);

        assert_eq!(
            result_str,
            r#"{"msg": "Hello World", "channel": "general"}"#
        );
    }

    #[test]
    fn test_resolve_auth_headers_bearer() {
        // SAFETY: Only running in single-threaded test context.
        // We set the env var temporarily for this test case.
        unsafe {
            env::set_var("TEST_TOKEN", "secret123");
        }
        let config = AuthConfig::Bearer {
            token_env: "TEST_TOKEN".to_string(),
        };
        let headers = resolve_auth_headers(&config);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer secret123");
    }

    #[test]
    fn test_apply_template_handlebars() {
        // Test {{json}} and {{eq}} properties
        let template = r#"{
            "system": {
                "parts": [ { "text": {{json system_instruction}} } ]
            },
            "is_user": {{#if (eq role "user")}}true{{else}}false{{/if}}
        }"#;

        // "abc" -> "\"abc\""
        let input = serde_json::json!({
            "system_instruction": "Be helpful.",
            "role": "user"
        });

        let result_str = apply_template(template, &input);

        let parsed: serde_json::Value =
            serde_json::from_str(&result_str).expect("Result should be valid JSON");
        assert_eq!(parsed["system"]["parts"][0]["text"], "Be helpful."); // Should be quoted in JSON
        assert_eq!(parsed["is_user"], true);

        // Test False condition
        let input2 = serde_json::json!({
            "system_instruction": "ignore",
            "role": "model"
        });
        let result_str2 = apply_template(template, &input2);
        let parsed2: serde_json::Value =
            serde_json::from_str(&result_str2).expect("Result should be valid JSON");
        assert_eq!(parsed2["is_user"], false);
    }

    #[test]
    fn test_resolve_auth_headers_basic() {
        // SAFETY: Only running in single-threaded test context.
        unsafe {
            env::set_var("TEST_USER", "user");
            env::set_var("TEST_PASS", "pass");
        }
        // "user:pass" base64 -> "dXNlcjpwYXNz"
        let config = AuthConfig::Basic {
            user_env: "TEST_USER".to_string(),
            pass_env: "TEST_PASS".to_string(),
        };
        let headers = resolve_auth_headers(&config);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Basic dXNlcjpwYXNz");
    }
    #[test]
    fn test_apply_template_gemini_advanced() {
        // Test is_string, is_array, and tool logic
        let template = r#"{
            "contents": [
                {{#each messages}}
                {
                    "role": "{{#if (eq role "user")}}user{{else}}model{{/if}}",
                    "parts": [
                        {{#if (is_string content)}}
                        { "text": {{json content}} }
                        {{else if (is_array content)}}
                        {{#each content}}{{json this}}{{#unless @last}},{{/unless}}{{/each}}
                        {{/if}}
                    ]
                }{{#unless @last}},{{/unless}}
                {{/each}}
            ],
            {{#if tools}}
            "tools": [{"function_declarations": {{json tools}}}],
            "tool_config": { "function_calling_config": { "mode": "{{#if (eq tool_choice "Required")}}ANY{{else}}AUTO{{/if}}" } }
            {{/if}}
        }"#;

        let input = json!({
            "messages": [
                { "role": "user", "content": "Hello" },
                { "role": "assistant", "content": [ { "text": "Hi" }, { "thought": "Thinking..." } ] }
            ],
            "tools": [ { "name": "test", "description": "desc", "parameters": {} } ],
            "tool_choice": "Auto"
        });

        let result_str = apply_template(template, &input);
        let parsed: Value = serde_json::from_str(&result_str).expect("Valid JSON expected");

        // Verify Roles
        assert_eq!(parsed["contents"][0]["role"], "user");
        assert_eq!(parsed["contents"][1]["role"], "model");

        // Verify String part
        assert_eq!(parsed["contents"][0]["parts"][0]["text"], "Hello");

        // Verify Array parts
        assert_eq!(parsed["contents"][1]["parts"][0]["text"], "Hi");
        assert_eq!(parsed["contents"][1]["parts"][1]["thought"], "Thinking...");

        // Verify Tools
        assert_eq!(
            parsed["tools"][0]["function_declarations"][0]["name"],
            "test"
        );
        assert_eq!(
            parsed["tool_config"]["function_calling_config"]["mode"],
            "AUTO"
        );
    }
}
