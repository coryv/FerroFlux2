use crate::domain::TenantId;
use crate::integrations::IntegrationRegistry;
use crate::store::database::PersistentStore;
use handlebars::Handlebars;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Live,
    DryRun,
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_integration_action(
    store: &PersistentStore,
    registry: &IntegrationRegistry,
    master_key: &[u8],
    tenant: &TenantId,
    slug: &str,
    action: &str,
    inputs: Option<Value>,
    mode: ExecutionMode,
    samples: Option<&std::collections::HashMap<String, Value>>,
) -> Result<String, String> {
    // 1. Load Connection
    let conn = store
        .get_connection_by_slug(tenant, slug)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Connection not found".to_string())?;

    let (provider_type, data, nonce, _, _) = conn;

    // 2. Decrypt
    let plaintext = crate::security::encryption::decrypt(&data, master_key, &nonce)
        .map_err(|e| format!("Decryption Failed: {}", e))?;

    let connection_fields: Value =
        serde_json::from_slice(&plaintext).map_err(|e| format!("Invalid JSON in DB: {}", e))?;

    // 3. Lookup Definition
    let def = registry
        .definitions
        .get(&provider_type)
        .ok_or_else(|| "Provider not found".to_string())?;

    let action_def = def
        .actions
        .get(action)
        .or_else(|| def.utilities.get(action))
        .or_else(|| def.resources.get(action))
        .ok_or_else(|| "Action not found".to_string())?;

    // 4. Handle Mock/DryRun Mode
    if mode == ExecutionMode::DryRun {
        if let Some(samples_map) = samples {
            // Check for a "default" sample or "success_200"
            // For now, prioritize "success_200", then "default", then the first available
            if let Some(sample) = samples_map.get("success_200") {
                return Ok(serde_json::to_string(sample).unwrap_or_default());
            }
            if let Some(sample) = samples_map.get("default") {
                return Ok(serde_json::to_string(sample).unwrap_or_default());
            }
            // Fallback: return first key
            if let Some((_, sample)) = samples_map.iter().next() {
                return Ok(serde_json::to_string(sample).unwrap_or_default());
            }
        }
        return Err("DryRun: No samples available for this node".to_string());
    }

    // 5. Prepare Execution (Handlebars)
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    // Helper for JSON encoding
    handlebars.register_helper(
        "json",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let param = h.param(0).ok_or(handlebars::RenderErrorReason::Other(
                    "Param 0 required".to_string(),
                ))?;
                out.write(&serde_json::to_string(param.value()).map_err(|e| {
                    handlebars::RenderErrorReason::Other(format!("JSON encode error: {}", e))
                })?)?;
                Ok(())
            },
        ),
    );

    // Context = Connection Fields + Inputs
    // We merge connection fields and inputs into a single object
    let mut context_map = serde_json::Map::new();

    // Add connection fields (e.g. api_key)
    if let Some(obj) = connection_fields.as_object() {
        context_map.extend(obj.clone());
    }

    // Add inputs (e.g. id, message)
    if let Some(inp) = inputs
        && let Some(obj) = inp.as_object()
    {
        context_map.extend(obj.clone());
    }

    let context = Value::Object(context_map);

    // Body
    let body_str = if let Some(tpl) = &action_def.implementation.config.body_template {
        handlebars
            .render_template(tpl, &context)
            .map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    // Path
    let path_str = handlebars
        .render_template(&action_def.implementation.config.path, &context)
        .map_err(|e| e.to_string())?;

    let url = format!("{}{}", def.base_url, path_str);

    // Headers
    let client = reqwest::Client::new();
    let method = match action_def.implementation.config.method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        _ => reqwest::Method::GET,
    };

    let mut request_builder = client.request(method, &url);

    for (k, v) in &action_def.implementation.config.headers {
        if let Ok(val) = handlebars.render_template(v, &context) {
            request_builder = request_builder.header(k, val);
        }
    }

    if !body_str.is_empty() {
        request_builder = request_builder.body(body_str);
    }

    // 6. Execute
    let resp = request_builder.send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Upstream error {}: {}", status, text));
    }

    let resp_text = resp.text().await.unwrap_or_default();

    // 7. Transform Output
    if let Some(transform) = &action_def.output_transform {
        let expr = jmespath::compile(&transform.text).map_err(|e| e.to_string())?;
        let data = jmespath::Variable::from_json(&resp_text).map_err(|_| resp_text.clone()); // Fallback if not JSON

        match data {
            Ok(d) => {
                let res = expr.search(&d).map_err(|e| e.to_string())?;
                Ok(serde_json::to_string(&res).unwrap_or(resp_text))
            }
            Err(raw) => Ok(raw),
        }
    } else {
        Ok(resp_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::{
        ActionImplementation, AuthType, IntegrationAction, IntegrationConfig, IntegrationDef,
    };
    use crate::store::database::PersistentStore;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_dry_run_execution() {
        // 1. Setup Registry
        let mut registry = IntegrationRegistry::default();
        let action_def = IntegrationAction {
            inputs: vec![],
            outputs: vec![],
            category: None,
            subcategory: None,
            documentation: None,
            message_transform: None,
            output_transform: None,
            implementation: ActionImplementation {
                impl_type: "http".to_string(),
                config: IntegrationConfig {
                    method: "GET".to_string(),
                    path: "/test".to_string(),
                    headers: HashMap::new(),
                    body_template: None,
                },
            },
        };

        let mut def = IntegrationDef {
            name: "test_provider".to_string(),
            base_url: "https://api.test.com".to_string(),
            icon_url: None,
            auth: None,
            connection_schema: None,
            verify_params: HashMap::new(),
            verify_endpoint: None,
            capabilities: None,
            actions: HashMap::new(),
            utilities: HashMap::new(),
            resources: HashMap::new(),
            auth_type: AuthType::None,
        };
        def.actions.insert("test_action".to_string(), action_def);
        registry
            .definitions
            .insert("test_provider".to_string(), def);

        // 2. Setup Store (Mock Connection)
        let store = PersistentStore::new("sqlite::memory:").await.unwrap();
        let tenant = TenantId::from("default_tenant");
        let master_key = vec![0u8; 32];

        // Save dummy connection
        let encrypted_data = crate::security::encryption::encrypt(b"{}", &master_key).unwrap();
        store
            .save_connection(
                &tenant,
                "test-conn",
                "Test Conn",
                "test_provider",
                &encrypted_data.0, // ciphertext
                &encrypted_data.1, // nonce
                "active",
            )
            .await
            .unwrap();

        // 3. Prepare Samples
        let mut samples = HashMap::new();
        let sample_data = serde_json::json!({ "id": "123", "status": "ok" });
        samples.insert("success_200".to_string(), sample_data.clone());

        // 4. Execute DryRun
        let result = execute_integration_action(
            &store,
            &registry,
            &master_key,
            &tenant,
            "test-conn",
            "test_action",
            None,
            ExecutionMode::DryRun,
            Some(&samples),
        )
        .await;

        assert!(result.is_ok());
        let json_resp = result.unwrap();
        assert_eq!(json_resp, sample_data.to_string());
    }

    #[tokio::test]
    async fn test_dry_run_no_samples() {
        // 1. Setup Registry & Store (Same as above)
        let mut registry = IntegrationRegistry::default();
        let action_def = IntegrationAction {
            inputs: vec![],
            outputs: vec![],
            category: None,
            subcategory: None,
            documentation: None,
            message_transform: None,
            output_transform: None,
            implementation: ActionImplementation {
                impl_type: "http".to_string(),
                config: IntegrationConfig {
                    method: "GET".to_string(),
                    path: "/test".to_string(),
                    headers: HashMap::new(),
                    body_template: None,
                },
            },
        };
        let mut def = IntegrationDef {
            name: "test_provider".to_string(),
            base_url: "https://api.test.com".to_string(),
            icon_url: None,
            auth: None,
            connection_schema: None,
            verify_params: HashMap::new(),
            verify_endpoint: None,
            capabilities: None,
            actions: HashMap::new(),
            utilities: HashMap::new(),
            resources: HashMap::new(),
            auth_type: AuthType::None,
        };
        def.actions.insert("test_action".to_string(), action_def);
        registry
            .definitions
            .insert("test_provider".to_string(), def);

        let store = PersistentStore::new("sqlite::memory:").await.unwrap();
        let tenant = TenantId::from("default_tenant");
        let master_key = vec![0u8; 32];
        let encrypted_data = crate::security::encryption::encrypt(b"{}", &master_key).unwrap();
        store
            .save_connection(
                &tenant,
                "test-conn",
                "Test Conn",
                "test_provider",
                &encrypted_data.0,
                &encrypted_data.1,
                "active",
            )
            .await
            .unwrap();

        // 2. Execute DryRun with NO samples
        let result = execute_integration_action(
            &store,
            &registry,
            &master_key,
            &tenant,
            "test-conn",
            "test_action",
            None,
            ExecutionMode::DryRun,
            None, // No map
        )
        .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "DryRun: No samples available for this node"
        );
    }
}
