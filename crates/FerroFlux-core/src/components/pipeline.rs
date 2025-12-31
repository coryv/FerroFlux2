use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Runtime component for a Node executing a pipeline.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNode {
    /// ID of the YAML definition this node is an instance of.
    pub definition_id: String,
    /// Configuration/Settings provided by the user (persisted).
    pub config: HashMap<String, Value>,
    /// Ephemeral state used during execution (not persisted usually, or handled specially).
    /// Used for "step results" during the pipeline run.
    #[serde(skip)]
    pub execution_context: HashMap<String, Value>,
}

impl PipelineNode {
    pub fn new(definition_id: String, config: HashMap<String, Value>) -> Self {
        Self {
            definition_id,
            config,
            execution_context: HashMap::new(),
        }
    }
}

// --- Restored Components ---

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub provider_name: String,
    pub model_name: String,
    pub node_id: uuid::Uuid,
    pub result_key: Option<String>,
    pub output_transform: Option<String>,
    pub input_json: Value,
    pub start_time: u64,
    // Legacy fields I thought were there but actually arent used or are handled differently?
    // workflow_id and tenant_id were in my previous truncated version.
    // Let's keep them if they are useful or remove them if they break compilation initialization?
    // agent_prep.rs DOES NOT initialize workflow_id, so I must remove it or make it Option/Default.
    // Same for tenant_id.
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ReadyToExecute {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub trace_id: String,
    pub context: ExecutionContext,
}

#[derive(Component, Debug, Clone)]
pub struct ExecutionResult {
    pub status: u16,
    pub raw_body: String,
    pub trace_id: String,
    pub context: ExecutionContext,
}
