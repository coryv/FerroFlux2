use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for an ephemeral Compute Node (WASM/Sandboxed).
///
/// Compute nodes execute untrusted user scripts in a secure sandbox (e.g., Wasmtime).
/// They are used for data transformation, light logic, and custom algorithm execution
/// without exposing the host system.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ComputeConfig {
    /// The runtime environment identifier (e.g., "python-3.11", "js-quickjs").
    pub runtime: String,
    /// The raw source code to be executed.
    pub source_code: String,
    /// The name of the function to invoke (default: "main").
    pub entry_point: String,
}

impl Default for ComputeConfig {
    fn default() -> Self {
        Self {
            runtime: "js-quickjs".to_string(),
            source_code: "console.log('Hello from WASM');".to_string(),
            entry_point: "main".to_string(),
        }
    }
}
