use bevy_ecs::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for a Logic Switch Node (Logic).
///
/// Evaluates a small JavaScript expression to determine which output edge to follow.
/// The script should return a string matching an edge label.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SwitchConfig {
    /// The logic script (e.g., `return input.value > 10 ? "high" : "low"`).
    pub script: String,
}

/// Configuration for a lightweight Compute Node (Data Transformation).
///
/// Executes a JS script to modify the payload in-memory.
/// Unlike `ComputeConfig` (which spawns a full Wasm sandbox), this is intended
/// for simple mappings using a lighter runtime (if applicable) or just differentiation in role.
///
/// *Note: Currently both may use the same underlying Wasm engine.*
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScriptConfig {
    /// The transformation script.
    pub script: String,
    /// Field to write the result to.
    #[serde(default)]
    pub result_key: Option<String>,
}
