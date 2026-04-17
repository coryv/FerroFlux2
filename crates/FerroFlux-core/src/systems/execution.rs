use crate::integrations::IntegrationRegistry;
use crate::store::database::PersistentStore;
use crate::systems::io::templating::{cel_json_func, cel_value_to_string, json_to_cel};
use ferroflux_iam::TenantId;
use cel_interpreter::{Context, Program};
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
    let plaintext = ferroflux_security::encryption::decrypt(&data, master_key, &nonce)
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

    // 5. Prepare CEL Context
    // Namespace inputs separately so user-supplied values cannot shadow connection credentials.
    let mut cel_data = serde_json::Map::new();
    if let Some(obj) = connection_fields.as_object() {
        for (k, v) in obj {
            cel_data.insert(k.clone(), v.clone());
        }
    }
    if let Some(inp) = inputs {
        cel_data.insert("inputs".to_string(), inp);
    }
    let cel_json = Value::Object(cel_data);

    let eval = |tpl: &str, label: &str| -> Result<String, String> {
        let program = Program::compile(tpl).map_err(|e| format!("{label} compile error: {e}"))?;
        let mut ctx = Context::default();
        ctx.add_function("json", cel_json_func);
        if let Some(obj) = cel_json.as_object() {
            for (k, v) in obj {
                let _ = ctx.add_variable(k, json_to_cel(v.clone()));
            }
        }
        let result = program.execute(&ctx).map_err(|e| format!("{label} execution error: {e}"))?;
        Ok(cel_value_to_string(result))
    };

    // Body
    let body_str = if let Some(tpl) = &action_def.implementation.config.body_template {
        eval(tpl, "Body template")?
    } else {
        String::new()
    };

    // Path
    let path_str = eval(&action_def.implementation.config.path, "Path template")?;

    let url = format!("{}{}", def.base_url, path_str);

    // Validate URL against SSRF
    ferroflux_security::network::validate_url(&url).map_err(|e| format!("Security Validation Failed: {}", e))?;

    // Headers
    let client = reqwest::Client::new();
    let method = match action_def.implementation.config.method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "PATCH" => reqwest::Method::PATCH,
        "DELETE" => reqwest::Method::DELETE,
        "HEAD" => reqwest::Method::HEAD,
        other => return Err(format!("Unsupported HTTP method: {other}")),
    };

    let mut request_builder = client.request(method, &url);

    for (k, v) in &action_def.implementation.config.headers {
        let val = eval(v, &format!("Header '{k}'"))?;
        request_builder = request_builder.header(k, val);
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
                    path: "'/test'".to_string(), // CEL literal
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
        let encrypted_data = ferroflux_security::encryption::encrypt(b"{}", &master_key).unwrap();
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
                    path: "'/test'".to_string(),
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
        let encrypted_data = ferroflux_security::encryption::encrypt(b"{}", &master_key).unwrap();
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
