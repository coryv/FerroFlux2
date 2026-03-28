use ferroflux_types::DataRef;
use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

use super::request::{
    build_multipart_form, check_ssrf, execute_binary_request, execute_multipart_request,
    execute_request, execute_streaming_request, resolve_connection_auth,
};

pub struct HttpClientTool;

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::DataRef;
    use ferroflux_types::BlobStore;
    use std::collections::HashMap;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    macro_rules! make_ctx {
        ($local:expr, $memory:expr, $masks:expr) => {
            ToolContext {
                local: $local,
                memory: $memory,
                trace_id: "test".to_string(),
                event_bus: None,
                shadow_mode: false,
                shadow_masks: $masks,
                store: None,
                secrets: None,
            }
        };
        ($local:expr, $memory:expr, $masks:expr, store: $store:expr) => {
            ToolContext {
                local: $local,
                memory: $memory,
                trace_id: "test".to_string(),
                event_bus: None,
                shadow_mode: false,
                shadow_masks: $masks,
                store: Some($store),
                secrets: None,
            }
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multipart_text_parts() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/upload", server_uri),
            "method": "POST",
            "body_type": "multipart",
            "parts": [
                {"name": "channel", "content": "C1234567"},
                {"name": "message", "content": "hello world"}
            ]
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multipart_blob_part() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        // Prepare BlobStore with binary data
        let store = BlobStore::default();
        let ticket = store.check_in(b"binary file content").unwrap();
        let server_uri = server.uri().to_string();

        let params = serde_json::json!({
            "url": format!("{}/upload", server_uri),
            "method": "POST",
            "body_type": "multipart",
            "parts": [
                {
                    "name": "file",
                    "content_var": "file_bytes",
                    "filename": "test.bin",
                    "content_type": "application/octet-stream"
                }
            ]
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            local.insert("file_bytes".to_string(), DataRef::Blob(ticket));
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks, store: &store);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multipart_content_json_part() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/upload"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/upload", server_uri),
            "method": "POST",
            "body_type": "multipart",
            "parts": [
                {
                    "name": "metadata",
                    "content_json": {"key": "value", "count": 42},
                    "content_type": "application/json"
                }
            ]
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        server.verify().await;
    }

    #[test]
    fn test_multipart_missing_parts_key() {
        let tool = HttpClientTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = make_ctx!(&mut local, &mut memory, &masks);

        // Provide a URL that won't be reached — error should happen before the request
        let params = serde_json::json!({
            "url": "http://example.com/upload",
            "method": "POST",
            "body_type": "multipart"
            // no "parts" key
        });

        // SSRF check runs first; bypass with env var
        // SAFETY: test-only, single-threaded
        unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(err.to_string().contains("parts"), "Expected 'parts' error, got: {}", err);
    }

    #[test]
    fn test_multipart_content_var_missing_from_context() {
        let tool = HttpClientTool;
        let mut local = HashMap::new(); // empty — no "file_bytes"
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = make_ctx!(&mut local, &mut memory, &masks);

        // SAFETY: test-only, single-threaded
        unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };

        let params = serde_json::json!({
            "url": "http://example.com/upload",
            "method": "POST",
            "body_type": "multipart",
            "parts": [
                {"name": "file", "content_var": "file_bytes"}
            ]
        });

        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(
            err.to_string().contains("file_bytes"),
            "Expected 'file_bytes' error, got: {}",
            err
        );
    }

    // ─── Binary body tests ───────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn test_binary_blob_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/audio"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        let store = BlobStore::default();
        let ticket = store.check_in(b"audio data bytes").unwrap();
        let server_uri = server.uri().to_string();

        let params = serde_json::json!({
            "url": format!("{}/audio", server_uri),
            "method": "POST",
            "body_type": "binary",
            "body_var": "audio_bytes",
            "content_type": "audio/mpeg"
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            local.insert("audio_bytes".to_string(), DataRef::Blob(ticket));
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks, store: &store);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_binary_inline_string_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/text"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/text", server_uri),
            "method": "POST",
            "body_type": "binary",
            "body_var": "raw_text",
            "content_type": "text/plain"
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            local.insert("raw_text".to_string(), DataRef::Inline(serde_json::json!("hello binary")));
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_binary_base64_body() {
        use base64::{Engine as _, engine::general_purpose};

        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/image"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .expect(1)
            .mount(&server)
            .await;

        let raw_bytes = b"PNG image data";
        let encoded = general_purpose::STANDARD.encode(raw_bytes);
        let server_uri = server.uri().to_string();

        let params = serde_json::json!({
            "url": format!("{}/image", server_uri),
            "method": "POST",
            "body_type": "binary",
            "body_base64": encoded,
            "content_type": "image/png"
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        server.verify().await;
    }

    #[test]
    fn test_binary_missing_content_type() {
        let tool = HttpClientTool;
        let mut local = HashMap::new();
        local.insert("data".to_string(), DataRef::Inline(serde_json::json!("bytes")));
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = make_ctx!(&mut local, &mut memory, &masks);

        // SAFETY: test-only, single-threaded
        unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };

        let params = serde_json::json!({
            "url": "http://example.com/upload",
            "method": "POST",
            "body_type": "binary",
            "body_var": "data"
            // no "content_type"
        });

        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(
            err.to_string().contains("content_type"),
            "Expected content_type error, got: {}",
            err
        );
    }

    #[test]
    fn test_binary_missing_body_source() {
        let tool = HttpClientTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = make_ctx!(&mut local, &mut memory, &masks);

        // SAFETY: test-only, single-threaded
        unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };

        let params = serde_json::json!({
            "url": "http://example.com/upload",
            "method": "POST",
            "body_type": "binary",
            "content_type": "application/octet-stream"
            // neither body_var nor body_base64
        });

        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(
            err.to_string().contains("body_var") || err.to_string().contains("body_base64"),
            "Expected body source error, got: {}",
            err
        );
    }

    // ─── Streaming SSE tests ─────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stream_openai_format() {
        let server = MockServer::start().await;

        // Simulate an OpenAI-style SSE stream body
        let sse_body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n",
            "data: [DONE]\n"
        );

        Mock::given(method("POST"))
            .and(path("/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_body),
            )
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/chat", server_uri),
            "method": "POST",
            "stream": true,
            "body": {"model": "gpt-4", "messages": [], "stream": true}
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        assert_eq!(result["body"]["text"], "Hello world");
        assert_eq!(result["body"]["total_chunks"], 2);
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stream_anthropic_format() {
        let server = MockServer::start().await;

        let sse_body = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n",
            "\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\" there\"}}\n",
            "\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n"
        );

        Mock::given(method("POST"))
            .and(path("/messages"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_body),
            )
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/messages", server_uri),
            "method": "POST",
            "stream": true,
            "body": {"model": "claude-3-5-sonnet-latest", "messages": [], "stream": true}
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        assert_eq!(result["body"]["text"], "Hi there");
        assert_eq!(result["body"]["total_chunks"], 3); // 2 delta + 1 stop event
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stream_gemini_format() {
        let server = MockServer::start().await;

        let sse_body = concat!(
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Gemini\"}]}}]}\n",
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\" streaming\"}]}}]}\n"
        );

        Mock::given(method("POST"))
            .and(path("/generate"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_body),
            )
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/generate", server_uri),
            "method": "POST",
            "stream": true
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        assert_eq!(result["body"]["text"], "Gemini streaming");
        assert_eq!(result["body"]["total_chunks"], 2);
        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stream_non_json_data_lines_skipped() {
        let server = MockServer::start().await;

        // Mix of valid JSON, non-JSON data lines, and [DONE]
        let sse_body = concat!(
            ": keep-alive\n",
            "data: not-json\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n",
            "data: [DONE]\n"
        );

        Mock::given(method("POST"))
            .and(path("/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(sse_body),
            )
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/chat", server_uri),
            "method": "POST",
            "stream": true
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only env var, single-threaded spawn
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = HttpClientTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["status"], 200);
        assert_eq!(result["body"]["text"], "ok");
        assert_eq!(result["body"]["total_chunks"], 1); // non-JSON line skipped
        server.verify().await;
    }
}

/// Resolves binary body bytes from params + context.
///
/// Accepts one of:
/// - `body_var`: variable name in `context.local`; resolved from `DataRef::Blob` (BlobStore claim)
///   or `DataRef::Inline` (string as UTF-8, other JSON values serialized)
/// - `body_base64`: inline base64-encoded string, decoded to raw bytes
fn resolve_binary_body(params: &Value, context: &mut ToolContext) -> Result<Vec<u8>> {
    if let Some(var_name) = params.get("body_var").and_then(|v| v.as_str()) {
        let data_ref = context
            .local
            .get(var_name)
            .ok_or_else(|| anyhow!("body_var '{}' not found in context", var_name))?;

        let bytes = match data_ref {
            DataRef::Blob(ticket) => {
                let store = context
                    .store
                    .ok_or_else(|| anyhow!("BlobStore not available to resolve body_var '{}'", var_name))?;
                store
                    .claim(ticket)
                    .with_context(|| format!("Failed to claim blob for body_var '{}'", var_name))?
            }
            DataRef::Inline(Value::String(s)) => s.as_bytes().to_vec(),
            DataRef::Inline(other) => serde_json::to_vec(other)
                .with_context(|| format!("Failed to serialize inline value for body_var '{}'", var_name))?,
        };

        return Ok(bytes);
    }

    if let Some(b64) = params.get("body_base64").and_then(|v| v.as_str()) {
        return general_purpose::STANDARD
            .decode(b64)
            .context("Failed to decode body_base64");
    }

    Err(anyhow!("Binary body requires either 'body_var' or 'body_base64'"))
}

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

        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url'"))?;

        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let body = params.get("body");
        let headers_val = params.get("headers");
        let connection_slug = params.get("connection").and_then(|v| v.as_str());

        // 1. Connection Resolution (Auth + Base URL)
        let (resolved_url, dynamic_headers) =
            resolve_connection_auth(url, connection_slug, context)?;

        // 2. SSRF Protection
        check_ssrf(&resolved_url)?;

        // 3. Execute — branch on stream or body_type
        let stream = params.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
        let body_type = params.get("body_type").and_then(|v| v.as_str()).unwrap_or("json");
        let client = reqwest::blocking::Client::new();

        if stream {
            let step_id = params
                .get("step_id")
                .and_then(|v| v.as_str())
                .unwrap_or("http_client");
            let (status, headers, body_val) = execute_streaming_request(
                &client,
                &resolved_url,
                method,
                headers_val,
                &dynamic_headers,
                body,
                context.event_bus.as_ref(),
                &context.trace_id,
                step_id,
            )?;
            return Ok(serde_json::json!({
                "status": status,
                "headers": headers,
                "body": body_val
            }));
        }

        let (status, headers, body_val) = if body_type == "multipart" {
            let parts = params
                .get("parts")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow!("Missing 'parts' array for multipart body"))?;
            let form = build_multipart_form(parts, context)?;
            execute_multipart_request(&client, &resolved_url, method, headers_val, &dynamic_headers, form)?
        } else if body_type == "binary" {
            let content_type = params
                .get("content_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing 'content_type' for binary body"))?;
            let bytes = resolve_binary_body(&params, context)?;
            execute_binary_request(&client, &resolved_url, method, headers_val, &dynamic_headers, bytes, content_type)?
        } else {
            execute_request(&client, &resolved_url, method, headers_val, &dynamic_headers, body)?
        };

        Ok(serde_json::json!({
            "status": status,
            "headers": headers,
            "body": body_val
        }))
    }
}
