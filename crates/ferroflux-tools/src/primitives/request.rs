//! Shared HTTP request utilities used by `http_client` and `paginate` tools.
//!
//! Provides connection auth resolution, SSRF protection, and request execution
//! as standalone functions so both tools can share the logic without duplication.

use ferroflux_types::events::{SystemEvent, SystemEventBus};
use ferroflux_types::DataRef;
use ferroflux_types::tool::ToolContext;
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ferroflux_iam::TenantId;
use std::collections::HashMap;
pub use super::aws_sigv4::{self, AwsSigV4Config};
use url::Url;
use std::io::{BufRead, BufReader};

use ipnet::IpNet;
use serde_json::Value;
use std::env;
use std::net::ToSocketAddrs;


/// Resolves an optional connection slug to auth headers and a fully-qualified URL.
///
/// Handles all auth types: Bearer, Basic, Custom Scheme, and OAuth2 (with token refresh).
/// Returns `(resolved_url, dynamic_auth_headers)`.
pub fn resolve_connection_auth(
    url: &str,
    connection_slug: Option<&str>,
    context: &mut ToolContext,
) -> Result<(String, Vec<(String, String)>)> {
    let mut url_str = url.to_string();
    let mut dynamic_headers: Vec<(String, String)> = Vec::new();

    if let Some(slug) = connection_slug {
        if let Some(resolver) = context.secrets {
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
                    "OAuth2" => {
                        // For core, the SecretResolver implementation handles refresh locks internally
                        // if it's the CoreSecretResolver. Portable tools don't need to know.
                        dynamic_headers.push((
                            "Authorization".to_string(),
                            format!("Bearer {}", access_token_from_conn(&conn_data)?),
                        ));
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

    Ok((url_str, dynamic_headers))
}

/// Validates a URL against the SSRF blocklist.
///
/// Resolves the hostname to IP(s) and rejects any that fall in private/loopback ranges,
/// unless `FERROFLUX_ALLOW_INTERNAL_IPS=true` is set in the environment.
pub fn check_ssrf(url: &str) -> Result<()> {
    let allow_internal = env::var("FERROFLUX_ALLOW_INTERNAL_IPS").unwrap_or_default() == "true";
    if allow_internal {
        return Ok(());
    }

    let parsed_url = Url::parse(url).context("Invalid URL")?;
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

    Ok(())
}

/// Executes a blocking HTTP request and returns `(status, response_headers, body)`.
///
/// The body is parsed as JSON; if parsing fails it is returned as a JSON string.
pub fn execute_request(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    static_headers: Option<&Value>,
    dynamic_headers: &[(String, String)],
    body: Option<&Value>,
    aws_config: Option<&AwsSigV4Config>,
) -> Result<(u16, HashMap<String, String>, Value)> {

    let mut final_dynamic_headers = dynamic_headers.to_vec();

    if let Some(config) = aws_config {
        let body_bytes = body.map(serde_json::to_vec).transpose()?.unwrap_or_default();

        let sig_headers = aws_sigv4::generate_sigv4_headers(config, method, url, dynamic_headers, &body_bytes)?;
        final_dynamic_headers.extend(sig_headers);
    }

    let mut req = match method {

        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };

    // Static headers from params
    if let Some(h) = static_headers.and_then(|v| v.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                req = req.header(k, s);
            }
        }
    }

    // Dynamic auth headers from connection and AWS SigV4
    for (k, v) in &final_dynamic_headers {
        req = req.header(k, v);
    }


    if let Some(b) = body {
        req = req.json(b);
    }

    let resp = req.send().context("HTTP request failed")?;



    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Parse body: detect XML or JSON
    let (body_val, _raw_bytes) = parse_response_body(resp, &resp_headers)?;

    Ok((status, resp_headers, body_val))
}

pub fn execute_raw_request(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    static_headers: Option<&Value>,
    dynamic_headers: &[(String, String)],
    body: Option<&Value>,
    aws_config: Option<&AwsSigV4Config>,
) -> Result<(u16, HashMap<String, String>, Vec<u8>)> {
    let mut final_dynamic_headers = dynamic_headers.to_vec();

    if let Some(config) = aws_config {
        let body_bytes = body.map(serde_json::to_vec).transpose()?.unwrap_or_default();
        let sig_headers = aws_sigv4::generate_sigv4_headers(config, method, url, dynamic_headers, &body_bytes)?;
        final_dynamic_headers.extend(sig_headers);
    }

    let mut req = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };

    if let Some(h) = static_headers.and_then(|v| v.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                req = req.header(k, s);
            }
        }
    }

    for (k, v) in &final_dynamic_headers {
        req = req.header(k, v);
    }

    if let Some(b) = body {
        req = req.json(b);
    }

    let mut resp = req.send().context("Raw HTTP request failed")?;

    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let mut bytes = Vec::new();
    resp.copy_to(&mut bytes).context("Failed to read response bytes")?;

    Ok((status, resp_headers, bytes))
}

fn parse_response_body(mut resp: reqwest::blocking::Response, headers: &HashMap<String, String>) -> Result<(Value, Vec<u8>)> {
    let mut bytes = Vec::new();
    resp.copy_to(&mut bytes).context("Failed to read response body")?;
    
    let text = String::from_utf8_lossy(&bytes);
    let body_val: Value = if headers
        .get("content-type")
        .map(|s| s.to_lowercase().contains("xml"))
        .unwrap_or(false)
    {
        parse_xml_to_json(&text).unwrap_or_else(|_| Value::String(text.to_string()))
    } else {
        serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text.to_string()))
    };

    Ok((body_val, bytes))
}


fn parse_xml_to_json(xml: &str) -> Result<Value> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<Value> = vec![serde_json::json!({})];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                stack.push(serde_json::json!({ "tag": name, "content": {} }));
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape()?.to_string();
                if let Some(parent) = stack.last_mut() {
                    if let Some(obj) = parent.as_object_mut() {
                        obj.insert("text".to_string(), Value::String(text));
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child = stack.pop().unwrap_or(Value::Null);
                if let Some(parent) = stack.last_mut() {
                    if let Some(obj) = parent.as_object_mut() {
                        let content = obj.get_mut("content").and_then(|v| v.as_object_mut()).unwrap();
                        let current = content.get(&name);
                        match current {
                            Some(Value::Array(arr)) => {
                                let mut new_arr = arr.clone();
                                new_arr.push(child);
                                content.insert(name, Value::Array(new_arr));
                            }
                            Some(val) => {
                                content.insert(name, Value::Array(vec![val.clone(), child]));
                            }
                            None => {
                                content.insert(name, child);
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => (),
        }
        buf.clear();
    }

    Ok(stack.pop().unwrap_or(Value::Null))
}


/// Extracts a value from a JSON document using a flexible path syntax.
///
/// Supports:
/// - JSON Pointer (`/foo/bar`) via `serde_json::Value::pointer()`
/// - Dot notation (`foo.bar`) — splits on `.` and descends
/// - Bare field name (`foo`) — looks up top-level key
///
/// Returns `None` if the path is empty or not found.
pub fn extract_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }

    // JSON Pointer
    if path.starts_with('/') {
        return value.pointer(path);
    }

    // Dot notation or bare field
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// Appends or replaces a query parameter on an existing URL string.
pub fn set_query_param(url: &str, key: &str, value: &str) -> Result<String> {
    let mut parsed = Url::parse(url).context("Invalid URL for query param injection")?;

    // Collect existing params, filtering out the key we're about to set
    let existing: Vec<(String, String)> = parsed
        .query_pairs()
        .filter(|(k, _)| k.as_ref() != key)
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    {
        let mut pairs = parsed.query_pairs_mut();
        pairs.clear();
        for (k, v) in &existing {
            pairs.append_pair(k, v);
        }
        pairs.append_pair(key, value);
    }
        Ok(parsed.to_string())
}

fn tenant_id_from_context(_context: &ToolContext) -> TenantId {
    TenantId::from("default_tenant")
}

fn access_token_from_conn(conn_data: &Value) -> Result<String> {
    conn_data
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("OAuth2 access_token missing in connection data"))
}

/// Parses the `Link: <url>; rel="next"` header and returns the next URL if present.
pub fn parse_link_header_next(headers: &HashMap<String, String>) -> Option<String> {
    // The Link header key may be lowercase after normalization
    let link = headers.get("link").or_else(|| headers.get("Link"))?;

    // Format: `<https://...>; rel="next", <https://...>; rel="prev"`
    for part in link.split(',') {
        let part = part.trim();
        let segments: Vec<&str> = part.splitn(2, ';').collect();
        if segments.len() != 2 {
            continue;
        }
        let url_part = segments[0].trim().trim_start_matches('<').trim_end_matches('>');
        let rel_part = segments[1].trim();
        if rel_part.contains("rel=\"next\"") || rel_part.contains("rel='next'") {
            return Some(url_part.to_string());
        }
    }
    None
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

    #[test]
    fn test_parse_link_header_next_single_quotes() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "<https://api.example.com/items?page=2>; rel='next'".to_string());
        assert_eq!(
            parse_link_header_next(&headers),
            Some("https://api.example.com/items?page=2".to_string())
        );
    }

    #[test]
    fn test_parse_link_header_next_multiple() {
        let mut headers = HashMap::new();
        headers.insert(
            "link".to_string(),
            "<https://api.example.com/items?page=1>; rel=\"prev\", <https://api.example.com/items?page=3>; rel=\"next\"".to_string()
        );
        assert_eq!(
            parse_link_header_next(&headers),
            Some("https://api.example.com/items?page=3".to_string())
        );
    }

    #[test]
    fn test_parse_link_header_next_case_insensitive_key() {
        let mut headers = HashMap::new();
        headers.insert("Link".to_string(), "<https://api.example.com/items?page=2>; rel=\"next\"".to_string());
        assert_eq!(
            parse_link_header_next(&headers),
            Some("https://api.example.com/items?page=2".to_string())
        );
    }

    #[test]
    fn test_parse_link_header_next_missing_next() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "<https://api.example.com/items?page=1>; rel=\"prev\"".to_string());
        assert_eq!(parse_link_header_next(&headers), None);
    }

    #[test]
    fn test_parse_link_header_next_malformed_part() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "<https://api.example.com/items?page=2>".to_string());
        assert_eq!(parse_link_header_next(&headers), None);
    }

    #[test]
    fn test_parse_link_header_next_empty_header() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "".to_string());
        assert_eq!(parse_link_header_next(&headers), None);
    }

    #[test]
    fn test_parse_link_header_next_no_header() {
        let headers = HashMap::new();
        assert_eq!(parse_link_header_next(&headers), None);
    }

    #[test]
    fn test_parse_link_header_next_extra_whitespace() {
        let mut headers = HashMap::new();
        headers.insert(
            "link".to_string(),
            "  <https://api.example.com/items?page=3>  ;  rel=\"next\"  ,  <https://api.example.com/items?page=1> ; rel=\"prev\" ".to_string()
        );
        assert_eq!(
            parse_link_header_next(&headers),
            Some("https://api.example.com/items?page=3".to_string())
        );
    }

    #[test]
    fn test_parse_link_header_next_no_brackets() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "https://api.example.com/items?page=2; rel=\"next\"".to_string());
        assert_eq!(
            parse_link_header_next(&headers),
            Some("https://api.example.com/items?page=2".to_string())
        );
    }

    #[test]
    fn test_parse_link_header_next_rel_contains_next_in_string() {
        let mut headers = HashMap::new();
        headers.insert("link".to_string(), "<https://api.example.com/items?page=2>; rel=\"not-next\"".to_string());
        assert_eq!(parse_link_header_next(&headers), None);

        // Note: the current implementation uses .contains("rel=\"next\"") which is quite specific.
        // It would NOT match rel="next-page" because "next-page\"" does not contain "next\"".
        headers.insert("link".to_string(), "<https://api.example.com/items?page=2>; rel=\"next-page\"".to_string());
        assert_eq!(parse_link_header_next(&headers), None);
    }
}

/// Builds a `multipart/form-data` form from a JSON `parts` array.
///
/// Each element in `parts` must have a `name` field and exactly one content source:
/// - `content`: literal text string
/// - `content_json`: any JSON value (serialized to string)
/// - `content_var`: variable name to look up in `context.local` (supports blobs)
///
/// Optional per-part fields: `filename`, `content_type`.
pub fn build_multipart_form(
    parts: &[Value],
    context: &ToolContext,
) -> Result<reqwest::blocking::multipart::Form> {
    let mut form = reqwest::blocking::multipart::Form::new();

    for part_val in parts {
        let name = part_val
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Multipart part missing required 'name' field"))?
            .to_string();

        let filename = part_val.get("filename").and_then(|v| v.as_str()).map(str::to_string);
        let mime_override = part_val.get("content_type").and_then(|v| v.as_str()).map(str::to_string);

        let mut part: reqwest::blocking::multipart::Part =
            if let Some(text) = part_val.get("content").and_then(|v| v.as_str()) {
                reqwest::blocking::multipart::Part::text(text.to_string())
            } else if let Some(json_val) = part_val.get("content_json") {
                let s = serde_json::to_string(json_val)
                    .context("Failed to serialize content_json")?;
                reqwest::blocking::multipart::Part::text(s)
            } else if let Some(var_name) = part_val.get("content_var").and_then(|v| v.as_str()) {
                let data_ref = context
                    .local
                    .get(var_name)
                    .ok_or_else(|| anyhow!("content_var '{}' not found in context.local", var_name))?;

                match data_ref {
                    DataRef::Inline(Value::String(s)) => {
                        reqwest::blocking::multipart::Part::text(s.clone())
                    }
                    DataRef::Inline(other) => {
                        let s = serde_json::to_string(other)
                            .context("Failed to serialize inline DataRef")?;
                        reqwest::blocking::multipart::Part::text(s)
                    }
                    DataRef::Blob(ticket) => {
                        let bytes = context
                            .store
                            .ok_or_else(|| anyhow!("BlobStore not available in ToolContext"))?
                            .claim(ticket)
                            .context("Failed to claim blob from BlobStore")?;

                        // Use explicit content_type, then ticket metadata, then nothing
                        let effective_mime = mime_override
                            .as_deref()
                            .or_else(|| ticket.metadata.get("content_type").map(|s| s.as_str()));

                        let mut p = reqwest::blocking::multipart::Part::bytes(bytes);
                        if let Some(m) = effective_mime {
                            p = p.mime_str(m).context("Invalid MIME type")?;
                        }
                        if let Some(ref f) = filename {
                            p = p.file_name(f.clone());
                        }
                        // Return early — filename/mime already applied for blobs
                        form = form.part(name, p);
                        continue;
                    }
                }
            } else {
                return Err(anyhow!(
                    "Part '{}' has no content source: provide 'content', 'content_json', or 'content_var'",
                    name
                ));
            };

        // Apply filename and content_type to non-blob parts
        if let Some(ref f) = filename {
            part = part.file_name(f.clone());
        }
        if let Some(ref m) = mime_override {
            part = part.mime_str(m).context("Invalid MIME type")?;
        }

        form = form.part(name, part);
    }

    Ok(form)
}

/// Executes a blocking HTTP request with a multipart/form-data body.
///
/// Shares header setup with `execute_request`; sends `.multipart(form)` instead of `.json(body)`.
pub fn execute_multipart_request(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    static_headers: Option<&Value>,
    dynamic_headers: &[(String, String)],
    form: reqwest::blocking::multipart::Form,
    aws_config: Option<&AwsSigV4Config>,
) -> Result<(u16, HashMap<String, String>, Value)> {
    // SigV4 for multipart is tricky because the body is complex.
    // However, S3 often uses raw binary PUT for uploads, which uses execute_binary_request.
    // For now, we'll implement it for binary and simple requests first.
    let final_dynamic_headers = dynamic_headers.to_vec();
    if let Some(_config) = aws_config {
        // Multipart signing is technically complex and requires different headers.
        // We will skip for now or implement as needed.
    }

    let mut req = match method {

        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => client.post(url), // multipart almost always POST/PUT
    };

    // Static headers from params
    if let Some(h) = static_headers.and_then(|v| v.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                req = req.header(k, s);
            }
        }
    }

    // Dynamic auth headers from connection and AWS SigV4
    for (k, v) in &final_dynamic_headers {
        req = req.header(k, v);
    }


    let resp = req.multipart(form).send().context("Multipart HTTP request failed")?;
    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let text = resp.text().unwrap_or_default();
    let body_val: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));

    Ok((status, resp_headers, body_val))
}

/// Executes a blocking HTTP request with a raw binary body.
///
/// Sets `Content-Type` to the provided MIME type and sends the raw bytes as the
/// request body. Use this for APIs that expect `audio/mpeg`, `image/png`,
/// `application/octet-stream`, etc.
pub fn execute_binary_request(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    static_headers: Option<&Value>,
    dynamic_headers: &[(String, String)],
    bytes: Vec<u8>,
    content_type: &str,
    aws_config: Option<&AwsSigV4Config>,
) -> Result<(u16, HashMap<String, String>, Value)> {
    let mut final_dynamic_headers = dynamic_headers.to_vec();

    if let Some(config) = aws_config {
        // MUST include content-type in signing if we're adding it manually
        let mut combined_headers = dynamic_headers.to_vec();
        combined_headers.push(("Content-Type".to_string(), content_type.to_string()));
        let sig_headers = aws_sigv4::generate_sigv4_headers(config, method, url, &combined_headers, &bytes)?;
        final_dynamic_headers.extend(sig_headers);
    }

    let mut req = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => client.post(url),
    };

    // Static headers from params
    if let Some(h) = static_headers.and_then(|v| v.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                req = req.header(k, s);
            }
        }
    }

    // Dynamic auth headers from connection and AWS SigV4
    for (k, v) in &final_dynamic_headers {
        req = req.header(k, v);
    }

    req = req.header("Content-Type", content_type).body(bytes);


    let resp = req.send().context("Binary HTTP request failed")?;
    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let text = resp.text().unwrap_or_default();
    let body_val: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));

    Ok((status, resp_headers, body_val))
}

/// Executes a streaming HTTP request and reads the response as Server-Sent Events (SSE).
///
/// Reads the response line-by-line using a `BufReader` (works because
/// `reqwest::blocking::Response` implements `std::io::Read`). For each `data:` line:
/// - Parses JSON and emits a `SystemEvent::StreamChunk` via the event bus
/// - Extracts text from common LLM formats (OpenAI, Anthropic, Gemini)
/// - Stops at `data: [DONE]` (OpenAI-style end marker) or end of stream
///
/// Returns `(status, headers, { "chunks": [...], "text": "...", "total_chunks": N })`.
/// `text` is the best-effort full-text accumulation from all chunks.
#[allow(clippy::too_many_arguments)]
pub fn execute_streaming_request(
    client: &reqwest::blocking::Client,
    url: &str,
    method: &str,
    static_headers: Option<&Value>,
    dynamic_headers: &[(String, String)],
    body: Option<&Value>,
    event_bus: Option<&SystemEventBus>,
    trace_id: &str,
    step_id: &str,
    aws_config: Option<&AwsSigV4Config>,
) -> Result<(u16, HashMap<String, String>, Value)> {
    let mut final_dynamic_headers = dynamic_headers.to_vec();

    if let Some(config) = aws_config {
        let body_bytes = body.map(serde_json::to_vec).transpose()?.unwrap_or_default();

        let sig_headers = aws_sigv4::generate_sigv4_headers(config, method, url, dynamic_headers, &body_bytes)?;
        final_dynamic_headers.extend(sig_headers);
    }

    let mut req = match method {

        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };

    if let Some(h) = static_headers.and_then(|v| v.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                req = req.header(k, s);
            }
        }
    }

    for (k, v) in &final_dynamic_headers {
        req = req.header(k, v);
    }


    if let Some(b) = body {
        req = req.json(b);
    }

    let resp = req.send().context("Streaming HTTP request failed")?;
    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let reader = BufReader::new(resp);
    let mut chunks: Vec<Value> = Vec::new();
    let mut text_parts: Vec<String> = Vec::new();

    for line in reader.lines() {
        let line = line.context("Error reading SSE stream")?;

        // Skip empty lines (SSE keep-alive / event separators) and non-data lines
        if !line.starts_with("data:") {
            continue;
        }

        let data = line["data:".len()..].trim();

        if data == "[DONE]" {
            if let Some(bus) = event_bus {
                let _ = bus.0.send(SystemEvent::StreamChunk {
                    trace_id: trace_id.to_string(),
                    step_id: step_id.to_string(),
                    chunk: String::new(),
                    done: true,
                });
            }
            break;
        }

        let parsed: Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => continue, // Skip non-JSON data lines
        };

        let text_chunk = extract_stream_text(&parsed);

        if let Some(bus) = event_bus {
            let _ = bus.0.send(SystemEvent::StreamChunk {
                trace_id: trace_id.to_string(),
                step_id: step_id.to_string(),
                chunk: text_chunk.clone().unwrap_or_default(),
                done: false,
            });
        }

        if let Some(t) = text_chunk {
            text_parts.push(t);
        }

        chunks.push(parsed);
    }

    let full_text = text_parts.join("");
    let total_chunks = chunks.len();
    let body_val = serde_json::json!({
        "chunks": chunks,
        "text": full_text,
        "total_chunks": total_chunks
    });

    Ok((status, resp_headers, body_val))
}

/// Extracts the text delta from common LLM streaming response chunk formats.
///
/// Tries (in order):
/// 1. **OpenAI / Azure OpenAI**: `choices[0].delta.content`
/// 2. **Anthropic**: `delta.text` (from `content_block_delta` events)
/// 3. **Google Gemini**: `candidates[0].content.parts[0].text`
///
/// Returns `None` if none of the paths match (e.g. metadata-only chunks).
fn extract_stream_text(chunk: &Value) -> Option<String> {
    // OpenAI / Azure OpenAI
    if let Some(s) = chunk
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("delta"))
        .and_then(|d| d.get("content"))
        .and_then(|v| v.as_str())
    {
        return Some(s.to_string());
    }

    // Anthropic (content_block_delta event)
    if let Some(s) = chunk
        .get("delta")
        .and_then(|d| d.get("text"))
        .and_then(|v| v.as_str())
    {
        return Some(s.to_string());
    }

    // Google Gemini
    if let Some(s) = chunk
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|v| v.as_str())
    {
        return Some(s.to_string());
    }

    None
}
