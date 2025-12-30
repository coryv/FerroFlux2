use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for referencing a loaded Integration Action.
///
/// Binds a generic "Integration Node" to a concrete provider and action
/// (e.g., Slack / Post Message).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Name of the integration (e.g., "slack")
    pub integration: String,
    /// Name of the action (e.g., "post_message")
    pub action: String,
    /// Optional overrides for auth env vars (e.g. per-node API keys).
    #[serde(default)]
    pub auth_override: HashMap<String, String>,
}

/// Dynamic Payload Mapping.
///
/// Transforms an input JSON payload into a format suitable for the external API.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PayloadMapper {
    /// Handlebars/Mustache template string.
    pub template: Option<String>,
    /// Static or dynamic headers to inject.
    pub headers: HashMap<String, String>,
}
