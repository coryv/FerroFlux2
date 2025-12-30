use anyhow::{Context, Result};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Definition of an input parameter for an integration action.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputDef {
    /// The input key name.
    pub name: String,
    /// JSON type expected (e.g. "string", "number").
    #[serde(rename = "type")]
    pub field_type: String,
    /// Is this input mandatory?
    #[serde(default)]
    pub required: bool,
    /// User-facing description.
    #[serde(default)]
    pub description: String,
    /// Is this field sensitive? (Mask in UI)
    #[serde(default)]
    pub is_secret: bool,
    /// Default value for the input
    #[serde(default)]
    pub default: Option<Value>,
    /// List of valid options (for dropdowns)
    #[serde(default)]
    pub options: Option<Vec<String>>,
    /// Action to call to get a list of valid options
    #[serde(default)]
    pub dynamic_source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputTransform {
    pub text: String,
    #[serde(default)]
    pub tool_calls: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntegrationCapabilities {
    #[serde(default)]
    pub chat: bool,
    #[serde(default)]
    pub tools: bool,
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub embedding: bool,
}

/// Configuration for the underlying HTTP request of an action.
#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationConfig {
    /// URL path suffix (appended to base_url).
    pub path: String,
    /// HTTP Method (GET, POST).
    pub method: String,
    /// Static headers to include.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Template for the request body.
    pub body_template: Option<String>,
}

/// Helper struct for the YAML schema to define implementation details.
#[derive(Debug, Clone, Deserialize)]
pub struct ActionImplementation {
    #[serde(alias = "type")]
    pub impl_type: String,
    pub config: IntegrationConfig,
}

/// A specific capability provided by an integration (e.g., "Send Message").
#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationAction {
    pub implementation: ActionImplementation,
    #[serde(default)]
    pub inputs: Vec<InputDef>,
    #[serde(default)]
    pub outputs: Vec<OutputDef>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    pub documentation: Option<String>,
    pub message_transform: Option<String>,
    pub output_transform: Option<OutputTransform>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum AuthDef {
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "api_key")]
    ApiKey {
        #[serde(default)]
        in_header: bool, // true = header, false = query param
        key_name: String, // e.g. "X-API-Key" or "api_key"
    },
    #[serde(rename = "oauth2")]
    OAuth2 {
        grant_type: String, // "authorization_code", "client_credentials"
        auth_url: Option<String>,
        token_url: Option<String>,
        scopes: Vec<String>,
    },
    #[serde(rename = "bearer")]
    Bearer,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum AuthType {
    #[serde(rename = "api_key")]
    ApiKey,
    #[serde(rename = "oauth2")]
    OAuth2,
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "none")]
    #[default]
    None,
}

/// Top-level definition of an external service integration.
#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationDef {
    /// Unique identifier/name (e.g. "slack").
    pub name: String,
    /// Base URL for API requests.
    pub base_url: String,
    /// Icon URL for UI
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Authentication mechanism definition.
    #[serde(default)]
    pub auth: Option<AuthDef>,
    /// Schema for the connection input (e.g. api_key, client_id).
    #[serde(default)]
    pub connection_schema: Option<Vec<InputDef>>,
    /// Map of available actions ("post_message", "get_user", etc.).
    pub actions: HashMap<String, IntegrationAction>,
    /// Map of supporting utility actions (hidden from palette, e.g. "list_models").
    #[serde(default)]
    pub utilities: HashMap<String, IntegrationAction>,
    /// Map of shared resources/capabilities (e.g. "chat", "embedding").
    #[serde(default)]
    pub resources: HashMap<String, IntegrationAction>,
    /// Authentication type for verification.
    #[serde(default)]
    pub auth_type: AuthType,
    /// Parameters for verification logic (e.g. {"api_key": "my_token_field"}).
    #[serde(default)]
    pub verify_params: HashMap<String, String>,
    /// Optional endpoint to hit for verification (e.g. "/auth.test" or "/models").
    #[serde(default)]
    pub verify_endpoint: Option<String>,
    /// Capabilities of this integration
    #[serde(default)]
    pub capabilities: Option<IntegrationCapabilities>,
}

#[derive(Resource, Default, Clone)]
pub struct IntegrationRegistry {
    pub definitions: HashMap<String, IntegrationDef>,
}

impl IntegrationRegistry {
    #[tracing::instrument(skip(self))]
    pub fn load_from_directory(&mut self, path: &str) -> Result<usize> {
        let dir_path = Path::new(path);
        if !dir_path.exists() {
            tracing::warn!(path = %path, "Integration directory does not exist");
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read integration file: {:?}", path))?;

                let def: IntegrationDef = serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse YAML: {:?}", path))?;

                tracing::info!(integration = %def.name, path = ?path, "Loaded integration");
                self.definitions.insert(def.name.clone(), def);
                count += 1;
            }
        }
        Ok(count)
    }
}
