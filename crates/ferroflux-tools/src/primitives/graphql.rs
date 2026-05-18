use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow, Context};
use serde_json::{Value, json};

pub struct GraphQlTool;

impl Tool for GraphQlTool {
    fn id(&self) -> &'static str {
        "core.utils.graphql"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let url = params.get("url").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'url'"))?;

        crate::primitives::request::check_ssrf(url)?;

        let query = params.get("query").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'query'"))?;
        let variables = params.get("variables").cloned().unwrap_or(json!({}));
        let operation_name = params.get("operation_name").and_then(|v| v.as_str());

        // Build the payload
        let mut body = json!({
            "query": query,
            "variables": variables,
        });
        if let Some(name) = operation_name {
            body.as_object_mut().unwrap().insert("operationName".to_string(), json!(name));
        }

        // We'll use a blocking client for simplicity in this primitive, 
        // consistent with other tools in this crate.
        let client = reqwest::blocking::Client::new();
        let mut request = client.post(url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "FerroFlux/1.0")
            .json(&body);

        // Add optional headers
        if let Some(headers) = params.get("headers").and_then(|v| v.as_object()) {
            for (key, val) in headers {
                if let Some(val_str) = val.as_str() {
                    request = request.header(key, val_str);
                }
            }
        }

        let response = request.send().context("Failed to send GraphQL request")?;
        let status = response.status().as_u16();
        let body: Value = response.json().context("Failed to parse GraphQL response as JSON")?;

        Ok(json!({
            "status": status,
            "data": body.get("data").cloned().unwrap_or(Value::Null),
            "errors": body.get("errors").cloned().unwrap_or(Value::Null),
            "raw": body
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_graphql_success() {
        let server = MockServer::start().await;
        let mock_response = json!({
            "data": {
                "user": { "id": "1", "name": "Test User" }
            }
        });

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .expect(1)
            .mount(&server)
            .await;

        let server_uri = server.uri();
        let params = json!({
            "url": format!("{}/graphql", server_uri),
            "query": "query GetUser($id: ID!) { user(id: $id) { id name } }",
            "variables": { "id": "1" }
        });

        let res = std::thread::spawn(move || {
            let tool = GraphQlTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = ToolContext {
                local: &mut local,
                memory: &mut memory,
                trace_id: "test".to_string(),
                node_id: "test_node".to_string(),
                tenant_id: "test_tenant".to_string(),
                event_bus: None,
                shadow_mode: false,
                shadow_masks: &masks,
                store: None,
                secrets: None, executor: None,
            };
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(res["status"], 200);
        assert_eq!(res["data"]["user"]["name"], "Test User");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_graphql_errors() {
        let server = MockServer::start().await;
        let mock_response = json!({
            "errors": [
                { "message": "Unauthorized access" }
            ]
        });

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&server)
            .await;

        let server_uri = server.uri();
        let params = json!({
            "url": server_uri,
            "query": "{ secretThing }"
        });

        let res = std::thread::spawn(move || {
            let tool = GraphQlTool;
            let mut local = HashMap::new();
            let mut memory = HashMap::new();
            let masks = HashMap::new();
            let mut ctx = ToolContext {
                local: &mut local,
                memory: &mut memory,
                trace_id: "test".to_string(),
                node_id: "test_node".to_string(),
                tenant_id: "test_tenant".to_string(),
                event_bus: None,
                shadow_mode: false,
                shadow_masks: &masks,
                store: None,
                secrets: None, executor: None,
            };
            tool.run(&mut ctx, params)
        })
        .join()
        .unwrap()
        .unwrap();

        assert_eq!(res["errors"][0]["message"], "Unauthorized access");
    }
}
