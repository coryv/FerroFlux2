//! Pagination Tool
//!
//! Automatically follows paginated API responses across multiple HTTP requests,
//! merging all results into a single array. Supports four common pagination strategies:
//!
//! - **cursor** / **page_token**: Extract a cursor/token from the response body and pass
//!   it as a query parameter on the next request.
//! - **offset**: Increment an offset query parameter by the page size each iteration.
//! - **link_header**: Follow the `Link: rel="next"` HTTP response header.

use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Context, Result, anyhow};
use serde_json::Value;

use super::request::{
    check_ssrf, execute_request, extract_by_path, parse_link_header_next, resolve_connection_auth,
    set_query_param, AwsSigV4Config, BlockingRequest,
};


const DEFAULT_MAX_PAGES: u64 = 100;
const DEFAULT_LIMIT: u64 = 100;

pub struct PaginateTool;

impl Tool for PaginateTool {
    fn id(&self) -> &'static str {
        "paginate"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        // Shadow Mode Interception
        if context.shadow_mode {
            let mock = context.shadow_masks.get(self.id());
            tracing::info!(
                tool = self.id(),
                mock_found = mock.is_some(),
                "Shadow Mode: Intercepting paginate request"
            );
            if let Some(cfg) = mock {
                if cfg.delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(cfg.delay_ms));
                }
                return Ok(cfg.return_value.clone());
            }
            return Ok(serde_json::json!({
                "items": [],
                "total_pages": 0,
                "total_items": 0
            }));
        }

        // --- Parse params ---

        let base_url = params
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
        let aws_auth = params.get("aws_auth");

        let aws_config = aws_auth.and_then(|v| {
            let service = v.get("service")?.as_str()?.to_string();
            let region = v.get("region")?.as_str()?.to_string();
            let access_key = v.get("access_key")?.as_str()?.to_string();
            let secret_key = v.get("secret_key")?.as_str()?.to_string();
            Some(AwsSigV4Config { service, region, access_key, secret_key })
        });


        let strategy = params
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("cursor");

        let results_field = params
            .get("results_field")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let cursor_field = params
            .get("cursor_field")
            .and_then(|v| v.as_str())
            .unwrap_or("next_cursor");

        let cursor_param = params
            .get("cursor_param")
            .and_then(|v| v.as_str())
            .unwrap_or("cursor");

        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_LIMIT);

        let limit_param = params
            .get("limit_param")
            .and_then(|v| v.as_str())
            .unwrap_or("limit");

        let offset_param = params
            .get("offset_param")
            .and_then(|v| v.as_str())
            .unwrap_or("offset");

        let max_pages = params
            .get("max_pages")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_MAX_PAGES);

        // --- Resolve auth once (same connection for all pages) ---
        let (resolved_base_url, dynamic_headers) =
            resolve_connection_auth(&base_url, connection_slug, context)?;

        let client = reqwest::blocking::Client::new();
        let mut all_items: Vec<Value> = Vec::new();
        let mut page_count: u64 = 0;

        // State for each strategy
        let mut current_url = resolved_base_url.clone();
        let mut cursor: Option<String> = None;
        let mut offset: u64 = 0;

        // For offset: inject initial limit/offset params
        if strategy == "offset" {
            current_url =
                set_query_param(&current_url, limit_param, &limit.to_string())
                    .context("Failed to inject limit param")?;
            current_url =
                set_query_param(&current_url, offset_param, "0")
                    .context("Failed to inject offset param")?;
        }

        loop {
            // Inject cursor/token on non-first pages
            let page_url = match strategy {
                "cursor" | "page_token" => {
                    if let Some(ref c) = cursor {
                        set_query_param(&current_url, cursor_param, c)
                            .context("Failed to inject cursor param")?
                    } else {
                        current_url.clone()
                    }
                }
                "offset" => current_url.clone(), // already has offset in URL
                "link_header" => current_url.clone(),
                unknown => return Err(anyhow!("Unknown pagination strategy: '{}'", unknown)),
            };

            // SSRF check on each page URL
            check_ssrf(&page_url)?;

            tracing::debug!(
                strategy,
                page = page_count + 1,
                url = %page_url,
                "Fetching page"
            );

            let req = BlockingRequest::new(&client, &page_url, method)
                .with_headers(headers_val, &dynamic_headers)
                .with_body(body)
                .with_aws(aws_config.as_ref());

            let (status, resp_headers, body_val) = execute_request(req)?;


            if status >= 400 {
                return Err(anyhow!(
                    "Pagination request failed on page {} with HTTP {}: {:?}",
                    page_count + 1,
                    status,
                    body_val
                ));
            }

            // Extract items from this page
            let page_items = if results_field.is_empty() {
                // Whole body is the array
                match &body_val {
                    Value::Array(arr) => arr.clone(),
                    _ => {
                        tracing::warn!(
                            "results_field is empty but response body is not an array — \
                             wrapping in single-element array"
                        );
                        vec![body_val.clone()]
                    }
                }
            } else {
                match extract_by_path(&body_val, results_field) {
                    Some(Value::Array(arr)) => arr.clone(),
                    Some(other) => {
                        tracing::warn!(
                            results_field,
                            "results_field points to a non-array value — wrapping"
                        );
                        vec![other.clone()]
                    }
                    None => {
                        tracing::warn!(results_field, "results_field not found in response body");
                        vec![]
                    }
                }
            };

            let page_item_count = page_items.len();
            all_items.extend(page_items);
            page_count += 1;

            // --- Check termination and advance state ---
            let has_next = match strategy {
                "cursor" | "page_token" => {
                    // Look for next cursor in body
                    let next = extract_by_path(&body_val, cursor_field)
                        .and_then(|v| match v {
                            Value::String(s) if !s.is_empty() => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        });
                    cursor = next.clone();
                    next.is_some()
                }
                "offset" => {
                    // Continue if we received a full page
                    if page_item_count < limit as usize {
                        false
                    } else {
                        offset += limit;
                        current_url =
                            set_query_param(&current_url, offset_param, &offset.to_string())
                                .context("Failed to advance offset")?;
                        true
                    }
                }
                "link_header" => {
                    match parse_link_header_next(&resp_headers) {
                        Some(next_url) => {
                            current_url = next_url;
                            true
                        }
                        None => false,
                    }
                }
                _ => false,
            };

            if !has_next {
                break;
            }

            if page_count >= max_pages {
                tracing::warn!(
                    max_pages,
                    total_items = all_items.len(),
                    "Reached max_pages limit — stopping pagination early"
                );
                break;
            }
        }

        let total_items = all_items.len() as u64;

        Ok(serde_json::json!({
            "items": all_items,
            "total_pages": page_count,
            "total_items": total_items
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    macro_rules! make_ctx {
        ($local:expr, $memory:expr, $masks:expr) => {
            ToolContext {
                local: $local,
                memory: $memory,
                trace_id: "test".to_string(),
                node_id: "test_node".to_string(),
                tenant_id: "test_tenant".to_string(),
                event_bus: None,
                shadow_mode: false,
                shadow_masks: $masks,
                store: None,
                secrets: None,
            }
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cursor_strategy_three_pages() {
        let server = MockServer::start().await;

        // Page 1
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"id": 1}, {"id": 2}],
                "next_cursor": "page2"
            })))
            .expect(1)
            .mount(&server)
            .await;

        // Page 2 — higher priority so it beats the generic page-1 mock
        Mock::given(method("GET"))
            .and(path("/items"))
            .and(query_param("cursor", "page2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"id": 3}, {"id": 4}],
                "next_cursor": "page3"
            })))
            .with_priority(1)
            .expect(1)
            .mount(&server)
            .await;

        // Page 3 — higher priority so it beats the generic page-1 mock
        Mock::given(method("GET"))
            .and(path("/items"))
            .and(query_param("cursor", "page3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"id": 5}],
                "next_cursor": null
            })))
            .with_priority(1)
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/items", server_uri),
            "strategy": "cursor",
            "results_field": "results",
            "cursor_field": "next_cursor",
            "cursor_param": "cursor",
            "max_pages": 10
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only, single-threaded spawn — no concurrent env reads
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = PaginateTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 5);
        assert_eq!(result["total_pages"], 3);
        assert_eq!(result["total_items"], 5);

        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_offset_strategy_partial_last_page() {
        let server = MockServer::start().await;

        // Full first page (3 items, limit=3)
        Mock::given(method("GET"))
            .and(path("/items"))
            .and(query_param("offset", "0"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"id": 1}, {"id": 2}, {"id": 3}]
            })))
            .expect(1)
            .mount(&server)
            .await;

        // Partial second page (2 items < limit=3 → stop)
        Mock::given(method("GET"))
            .and(path("/items"))
            .and(query_param("offset", "3"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"id": 4}, {"id": 5}]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/items", server_uri),
            "strategy": "offset",
            "results_field": "data",
            "limit": 3,
            "limit_param": "limit",
            "offset_param": "offset",
            "max_pages": 10
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only, single-threaded spawn — no concurrent env reads
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = PaginateTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 5);
        assert_eq!(result["total_pages"], 2);

        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_link_header_strategy() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([{"name": "repo1"}, {"name": "repo2"}]))
                    .append_header(
                        "Link",
                        format!("<{}/repos?page=2>; rel=\"next\"", server.uri()),
                    ),
            )
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/repos"))
            .and(query_param("page", "2"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([{"name": "repo3"}])),
            )
            .with_priority(1)
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/repos", server_uri),
            "strategy": "link_header",
            "results_field": "",   // whole body is the array
            "max_pages": 10
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only, single-threaded spawn — no concurrent env reads
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = PaginateTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(result["total_pages"], 2);

        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_max_pages_safety_limit() {
        let server = MockServer::start().await;

        // Always return a cursor — would paginate forever without max_pages
        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"id": 1}],
                "next_cursor": "always_more"
            })))
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/items", server_uri),
            "strategy": "cursor",
            "results_field": "results",
            "cursor_field": "next_cursor",
            "cursor_param": "cursor",
            "max_pages": 3
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only, single-threaded spawn — no concurrent env reads
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = PaginateTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["total_pages"], 3);
        assert_eq!(result["total_items"], 3); // 1 item per page × 3 pages
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_http_error_on_page_2() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/items"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"id": 1}],
                "next_cursor": "page2"
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/items"))
            .and(query_param("cursor", "page2"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": "Unauthorized"
            })))
            .with_priority(1)
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/items", server_uri),
            "strategy": "cursor",
            "results_field": "results",
            "cursor_field": "next_cursor",
            "cursor_param": "cursor",
            "max_pages": 10
        });

        let err = std::thread::spawn(move || {
            let tool = PaginateTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap_err();

        assert!(err.to_string().contains("401"), "Expected 401 error, got: {}", err);

        server.verify().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_page_token_strategy() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": "a"}],
                "nextPageToken": "tok2"
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/list"))
            .and(query_param("pageToken", "tok2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"id": "b"}],
                "nextPageToken": null
            })))
            .with_priority(1)
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri().to_string();
        let params = serde_json::json!({
            "url": format!("{}/list", server_uri),
            "strategy": "page_token",
            "results_field": "items",
            "cursor_field": "nextPageToken",
            "cursor_param": "pageToken",
            "max_pages": 10
        });

        let result = std::thread::spawn(move || {
            // SAFETY: test-only, single-threaded spawn — no concurrent env reads
            unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true") };
            let tool = PaginateTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = make_ctx!(&mut local, &mut memory, &masks);
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(result["total_items"], 2);
        assert_eq!(result["total_pages"], 2);

        server.verify().await;
    }
}
