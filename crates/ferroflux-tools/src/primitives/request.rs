use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ferroflux_types::tool::{ToolContext};
use ferroflux_types::events::SystemEventBus;
use serde_json::Value;
use std::collections::HashMap;

/// Resolves a connection slug into auth headers and a base URL.
pub fn resolve_connection_auth(
    url: &str,
    connection_slug: Option<&str>,
    context: &mut ToolContext,
) -> Result<(String, Vec<(String, String)>)> {
    let mut url_str = url.to_string();
    let mut dynamic_headers: Vec<(String, String)> = Vec::new();

    if let Some(slug) = connection_slug {
        let resolver = context.secrets.ok_or_else(|| anyhow::anyhow!("Connection '{}' provided but no secret resolver available", slug))?;
        let tenant = tenant_id_from_context(context);

        let conn_data = resolver
            .resolve_connection(&tenant, slug)
            .context("Failed to resolve connection")?;

        // Apply Base URL
        if let Some(base) = conn_data.get("base_url").and_then(|v| v.as_str()) {
            let base = base.trim_end_matches('/');
            let path = url_str.trim_start_matches('/');
            if path.is_empty() {
                url_str = base.to_string();
            } else if !url_str.starts_with("http") {
                url_str = format!("{}/{}", base, path);
            }
        }

        // Apply Auth
        if let Some(auth_type) = conn_data.get("auth_type").and_then(|v| v.as_str()) {
            match auth_type {
                "Bearer" => {
                    if let Some(cred) = conn_data.get("credentials").and_then(|v| v.as_str()) {
                        dynamic_headers.push(("Authorization".to_string(), format!("Bearer {}", cred)));
                    }
                }
                "Basic" => {
                    if let Some(cred) = conn_data.get("credentials").and_then(|v| v.as_str()) {
                        let encoded = general_purpose::STANDARD.encode(cred);
                        dynamic_headers.push(("Authorization".to_string(), format!("Basic {}", encoded)));
                    }
                }
                "Custom Scheme" => {
                    let scheme = conn_data.get("auth_scheme").and_then(|v| v.as_str()).unwrap_or("Bearer");
                    if let Some(cred) = conn_data.get("credentials").and_then(|v| v.as_str()) {
                        dynamic_headers.push(("Authorization".to_string(), format!("{} {}", scheme, cred)));
                    }
                }
                "OAuth2" => {
                    dynamic_headers.push((
                        "Authorization".to_string(),
                        format!("Bearer {}", access_token_from_conn(&conn_data)?),
                    ));
                }
                _ => {}
            }
        }
    } else if !url_str.starts_with("http") {
        anyhow::bail!("Relative URL '{}' provided but no connection specified to resolve base URL", url_str);
    }

    Ok((url_str, dynamic_headers))
}

fn tenant_id_from_context(context: &ToolContext) -> ferroflux_iam::TenantId {
    ferroflux_iam::TenantId::from(context.tenant_id.as_str())
}

fn access_token_from_conn(conn: &Value) -> Result<String> {
    conn.get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing 'access_token' in OAuth2 connection"))
}

/// Simple SSRF check - prevents access to internal IP ranges.
pub fn check_ssrf(url: &str) -> Result<()> {
    if std::env::var("FERROFLUX_ALLOW_INTERNAL_IPS").is_ok() {
        return Ok(());
    }

    ferroflux_security::network::validate_url(url)
        .map_err(|e| anyhow::anyhow!("SSRF Protection blocked request: {}", e))
}

pub struct BlockingRequest<'a> {
    pub client: &'a reqwest::blocking::Client,
    pub url: &'a str,
    pub method: &'a str,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<Value>,
    pub aws: Option<&'a AwsSigV4Config>,
}

pub struct AwsSigV4Config {
    pub service: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl<'a> BlockingRequest<'a> {
    pub fn new(client: &'a reqwest::blocking::Client, url: &'a str, method: &'a str) -> Self {
        Self {
            client,
            url,
            method,
            headers: reqwest::header::HeaderMap::new(),
            body: None,
            aws: None,
        }
    }

    pub fn with_headers(mut self, user_headers: Option<&Value>, dynamic: &[(String, String)]) -> Self {
        if let Some(Value::Object(map)) = user_headers {
            for (k, v) in map {
                if let Some(s) = v.as_str()
                    && let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                    && let Ok(val) = reqwest::header::HeaderValue::from_str(s)
                {
                    self.headers.insert(name, val);
                }
            }
        }
        for (k, v) in dynamic {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                && let Ok(val) = reqwest::header::HeaderValue::from_str(v)
            {
                self.headers.insert(name, val);
            }
        }
        self
    }

    pub fn with_body(mut self, body: Option<&Value>) -> Self {
        self.body = body.cloned();
        self
    }

    pub fn with_aws(mut self, aws: Option<&'a AwsSigV4Config>) -> Self {
        self.aws = aws;
        self
    }
}

pub fn execute_request(req: BlockingRequest) -> Result<(u16, HashMap<String, String>, Value)> {
    let method = reqwest::Method::from_bytes(req.method.as_bytes()).context("Invalid method")?;
    let mut builder = req.client.request(method, req.url).headers(req.headers);

    if let Some(body) = req.body {
        builder = builder.json(&body);
    }

    let resp = builder.send().context("Request failed")?;
    let status = resp.status().as_u16();
    let mut headers = HashMap::new();
    for (name, value) in resp.headers() {
        headers.insert(name.to_string(), value.to_str().unwrap_or_default().to_string());
    }

    let body: Value = resp.json().unwrap_or(Value::Null);
    Ok((status, headers, body))
}

pub fn execute_raw_request(req: BlockingRequest) -> Result<(u16, HashMap<String, String>, Vec<u8>)> {
    let method = reqwest::Method::from_bytes(req.method.as_bytes()).context("Invalid method")?;
    let mut builder = req.client.request(method, req.url).headers(req.headers);

    if let Some(body) = req.body {
        builder = builder.json(&body);
    }

    let resp = builder.send().context("Request failed")?;
    let status = resp.status().as_u16();
    let mut headers = HashMap::new();
    for (name, value) in resp.headers() {
        headers.insert(name.to_string(), value.to_str().unwrap_or_default().to_string());
    }

    let bytes = resp.bytes().context("Failed to read response body")?.to_vec();
    Ok((status, headers, bytes))
}

pub fn execute_binary_request(req: BlockingRequest, bytes: Vec<u8>, content_type: &str) -> Result<(u16, HashMap<String, String>, Value)> {
    let method = reqwest::Method::from_bytes(req.method.as_bytes()).context("Invalid method")?;
    let mut headers = req.headers;
    headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_str(content_type)?);
    
    let builder = req.client.request(method, req.url).headers(headers).body(bytes);
    let resp = builder.send().context("Request failed")?;
    let status = resp.status().as_u16();
    let mut headers_map = HashMap::new();
    for (name, value) in resp.headers() {
        headers_map.insert(name.to_string(), value.to_str().unwrap_or_default().to_string());
    }

    let body: Value = resp.json().unwrap_or(Value::Null);
    Ok((status, headers_map, body))
}

pub fn execute_multipart_request(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    headers_val: Option<&Value>,
    dynamic_headers: &[(String, String)],
    form: reqwest::blocking::multipart::Form,
    _aws: Option<&AwsSigV4Config>,
) -> Result<(u16, HashMap<String, String>, Value)> {
    let method = reqwest::Method::from_bytes(method.as_bytes()).context("Invalid method")?;
    let mut req_headers = reqwest::header::HeaderMap::new();
    
    if let Some(Value::Object(map)) = headers_val {
        for (k, v) in map {
            if let Some(s) = v.as_str()
                && let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                && let Ok(val) = reqwest::header::HeaderValue::from_str(s)
            {
                req_headers.insert(name, val);
            }
        }
    }
    for (k, v) in dynamic_headers {
        if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes())
            && let Ok(val) = reqwest::header::HeaderValue::from_str(v)
        {
            req_headers.insert(name, val);
        }
    }

    let builder = client.request(method, url).headers(req_headers).multipart(form);
    let resp = builder.send().context("Multipart request failed")?;
    let status = resp.status().as_u16();
    let mut resp_headers = HashMap::new();
    for (name, value) in resp.headers() {
        resp_headers.insert(name.to_string(), value.to_str().unwrap_or_default().to_string());
    }

    let body: Value = resp.json().unwrap_or(Value::Null);
    Ok((status, resp_headers, body))
}

pub fn build_multipart_form(parts: &[Value], context: &mut ToolContext) -> Result<reqwest::blocking::multipart::Form> {
    let mut form = reqwest::blocking::multipart::Form::new();
    for part in parts {
        let name = part.get("name").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Part missing 'name'"))?;
        
        let mut form_part = if let Some(content) = part.get("content").and_then(|v| v.as_str()) {
            reqwest::blocking::multipart::Part::text(content.to_string())
        } else if let Some(content_json) = part.get("content_json") {
            reqwest::blocking::multipart::Part::text(serde_json::to_string(content_json)?)
        } else if let Some(var_name) = part.get("content_var").and_then(|v| v.as_str()) {
            let data_ref = context.local.get(var_name).ok_or_else(|| anyhow::anyhow!("content_var '{}' not found", var_name))?;
            let bytes = match data_ref {
                ferroflux_types::DataRef::Blob(ticket) => {
                    let store = context.store.ok_or_else(|| anyhow::anyhow!("BlobStore not available"))?;
                    store.claim(ticket)?
                }
                ferroflux_types::DataRef::Inline(Value::String(s)) => s.as_bytes().to_vec(),
                ferroflux_types::DataRef::Inline(other) => serde_json::to_vec(other)?,
            };
            reqwest::blocking::multipart::Part::bytes(bytes)
        } else {
            anyhow::bail!("Part '{}' missing content", name);
        };

        if let Some(filename) = part.get("filename").and_then(|v| v.as_str()) {
            form_part = form_part.file_name(filename.to_string());
        }
        if let Some(mime) = part.get("content_type").and_then(|v| v.as_str()) {
            form_part = form_part.mime_str(mime)?;
        }
        form = form.part(name.to_string(), form_part);
    }
    Ok(form)
}

pub fn execute_streaming_request(
    req: BlockingRequest,
    _bus: Option<&SystemEventBus>,
    _trace_id: &str,
    _step_id: &str,
) -> Result<(u16, HashMap<String, String>, Value)> {
    let method = reqwest::Method::from_bytes(req.method.as_bytes()).context("Invalid method")?;
    let mut builder = req.client.request(method, req.url).headers(req.headers);

    if let Some(body) = req.body {
        builder = builder.json(&body);
    }

    let resp = builder.send().context("Streaming request failed")?;
    let status = resp.status().as_u16();
    let mut headers = HashMap::new();
    for (name, value) in resp.headers() {
        headers.insert(name.to_string(), value.to_str().unwrap_or_default().to_string());
    }

    if status >= 400 {
        let body: Value = resp.json().unwrap_or(Value::Null);
        return Ok((status, headers, body));
    }

    // SSE Aggregation Logic
    let mut full_text = String::new();
    let mut chunk_count = 0;
    
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(resp);
    
    for line in reader.lines() {
        let line = line?;
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<Value>(data) {
                chunk_count += 1;
                // Try OpenAI format: choices[0].delta.content
                if let Some(content) = val.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
                    full_text.push_str(content);
                } 
                // Try Anthropic format: delta.text
                else if let Some(content) = val.pointer("/delta/text").and_then(|v| v.as_str()) {
                    full_text.push_str(content);
                }
                // Try Gemini format: candidates[0].content.parts[0].text
                else if let Some(content) = val.pointer("/candidates/0/content/parts/0/text").and_then(|v| v.as_str()) {
                    full_text.push_str(content);
                }
            }
        }
    }

    Ok((status, headers, serde_json::json!({
        "text": full_text,
        "total_chunks": chunk_count
    })))
}

pub fn set_query_param(url: &str, key: &str, value: &str) -> Result<String> {
    let mut parsed = url::Url::parse(url).context("Invalid URL for query param")?;
    let pairs: Vec<_> = parsed.query_pairs()
        .filter(|(k, _)| k != key)
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    
    parsed.query_pairs_mut().clear().extend_pairs(pairs).append_pair(key, value);
    Ok(parsed.to_string())
}

pub fn parse_link_header_next(headers: &HashMap<String, String>) -> Option<String> {
    let link = headers.get("link").or_else(|| headers.get("Link"))?;
    for part in link.split(',') {
        if part.contains("rel=\"next\"") || part.contains("rel='next'") {
            let start = part.find('<')? + 1;
            let end = part.find('>')?;
            return Some(part[start..end].to_string());
        }
    }
    None
}

/// Helper to extract value from nested JSON by dot-path
pub fn extract_by_path(value: &Value, path: &str) -> Option<Value> {
    if path.is_empty() {
        return Some(value.clone());
    }
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(part)?,
            Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                arr.get(idx)?
            }
            _ => return None,
        };
    }
    Some(current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_link_header_next_basic() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "<https://api.example.com/items?page=2>; rel=\"next\"".to_string());
        assert_eq!(
            parse_link_header_next(&headers),
            Some("https://api.example.com/items?page=2".to_string())
        );
    }
}
