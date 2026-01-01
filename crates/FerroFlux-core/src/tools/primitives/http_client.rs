use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

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

            // Default mock response if none specified
            return Ok(serde_json::json!({
                "status": 200,
                "body": { "message": "Shadow Mode: Request Intercepted" },
                "headers": {}
            }));
        }

        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'url'"))?;
        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        let body = params.get("body");
        let headers_val = params.get("headers");

        // Use blocking client for now to match Sync trait signature
        let client = reqwest::blocking::Client::new();

        let mut req = match method {
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => client.get(url),
        };

        if let Some(h) = headers_val.and_then(|v| v.as_object()) {
            for (k, v) in h {
                if let Some(s) = v.as_str() {
                    req = req.header(k, s);
                }
            }
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
        let body_val: Value = resp.json().unwrap_or(Value::Null);

        Ok(serde_json::json!({
            "status": status,
            "headers": headers,
            "body": body_val
        }))
    }
}
