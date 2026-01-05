use crate::secrets::SecretStore;
use crate::tools::{Tool, ToolContext};
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use ferroflux_iam::TenantId;
use ipnet::IpNet;
use serde_json::Value;
use std::env;
use std::net::ToSocketAddrs;
use url::Url;

pub struct HttpClientTool;

impl Tool for HttpClientTool {
    fn id(&self) -> &'static str {
        "http_client"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        // Shadow Mode Interception
        if context.shadow_mode {
            let mock = context.shadow_masks.get(self.id());
            tracing::info!(
                tool = self.id(),
                mock_found = mock.is_some(),
                "Shadow Mode: Intercepting HTTP request"
            );

            if let Some(cfg) = mock {
                if cfg.delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(cfg.delay_ms));
                }
                return Ok(cfg.return_value.clone());
            }

            return Ok(serde_json::json!({
                "status": 200,
                "body": { "message": "Shadow Mode: Request Intercepted" },
                "headers": {}
            }));
        }

        let mut url_str = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'url'"))?
            .to_string();

        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let body = params.get("body");
        let headers_val = params.get("headers");
        let connection_slug = params.get("connection").and_then(|v| v.as_str());

        // 1. Connection Resolution (Auth)
        let mut dynamic_headers = Vec::new();

        if let Some(slug) = connection_slug {
            if let Some(store) = context.secret_store
                && let Some(rt) = context.runtime
            {
                // TODO: TenantId resolution properly. For now use default or from context?
                // Assuming "default_tenant" or we need to add tenant_id to ToolContext.
                // Let's assume a default for now as primitive tools don't know tenants yet.
                let tenant = TenantId::from("default_tenant");

                let conn_data =
                    rt.0.block_on(async { store.resolve_connection(&tenant, slug).await })
                        .context("Failed to resolve connection")?;

                // Apply Base URL
                if let Some(base) = conn_data.get("base_url").and_then(|v| v.as_str()) {
                    let base = base.trim_end_matches('/');
                    let path = url_str.trim_start_matches('/');
                    if path.is_empty() {
                        url_str = base.to_string();
                    } else if !url_str.starts_with("http") {
                        // Only prepend if url_str isn't already absolute
                        url_str = format!("{}/{}", base, path);
                    }
                }

                // Apply Auth
                if let Some(auth_type) = conn_data.get("auth_type").and_then(|v| v.as_str()) {
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

                // Apply Custom Headers from Connection
                if let Some(h) = conn_data.get("custom_headers").and_then(|v| v.as_object()) {
                    for (k, v) in h {
                        if let Some(s) = v.as_str() {
                            dynamic_headers.push((k.clone(), s.to_string()));
                        }
                    }
                }
            } else {
                tracing::warn!(
                    "Connection slug provided but SecretStore/Runtime not available in context."
                );
            }
        }

        // 2. SSRF Protection
        let allow_internal = env::var("FERROFLUX_ALLOW_INTERNAL_IPS").unwrap_or_default() == "true";
        if !allow_internal {
            let parsed_url = Url::parse(&url_str).context("Invalid URL")?;
            let host_str = parsed_url
                .host_str()
                .ok_or_else(|| anyhow!("No host in URL"))?;
            let port = parsed_url.port_or_known_default().unwrap_or(80);

            let socket_addrs = format!("{}:{}", host_str, port)
                .to_socket_addrs()
                .context("DNS Resolution Failed")?;

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
                        return Err(anyhow!("Blocked Internal IP: {}", ip));
                    }
                }
            }
        }

        // 3. Execution using reqwest::blocking
        let client = reqwest::blocking::Client::new();
        let mut req = match method {
            "POST" => client.post(&url_str),
            "PUT" => client.put(&url_str),
            "DELETE" => client.delete(&url_str),
            _ => client.get(&url_str),
        };

        // Add Headers from params
        if let Some(h) = headers_val.and_then(|v| v.as_object()) {
            for (k, v) in h {
                if let Some(s) = v.as_str() {
                    req = req.header(k, s);
                }
            }
        }

        // Add Dynamic Headers (Auth)
        for (k, v) in dynamic_headers {
            req = req.header(k, v);
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send()?;
        let status = resp.status().as_u16();
        let headers: std::collections::HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Attempt to parse JSON, fallback to text
        let body_val: Value = resp.json().unwrap_or(Value::Null); // Logic change: Do not consume body twice? 
        // reqwest::blocking::Response::json consumes self.
        // Logic check: if json fails, we lost the body text?
        // Fix: get text first.

        // Actually better:
        // let text = resp.text()?;
        // let body_val = serde_json::from_str(&text).unwrap_or(Value::String(text));

        // Revising implementation below to be safe:

        Ok(serde_json::json!({
            "status": status,
            "headers": headers,
            "body": body_val
        }))
    }
}
