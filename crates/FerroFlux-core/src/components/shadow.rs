use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a mock response in Shadow Mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// The value to return when the tool is called.
    pub return_value: serde_json::Value,
    /// Optional simulated delay in milliseconds.
    #[serde(default)]
    pub delay_ms: u64,
}

/// Component that marks an entity as running in Shadow Mode.
///
/// When this component is present, dangerous tools (like HTTP clients) should
/// skip actual execution and return mocked data instead.
#[derive(Component, Debug, Clone, Default)]
pub struct ShadowExecution {
    /// Map of Tool Name -> Mock Configuration.
    /// e.g. "http_client" -> { return_value: { "status": 200 } }
    pub mocked_tools: HashMap<String, MockConfig>,
}
