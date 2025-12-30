use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Stage 1 -> Stage 2: Ready to hit the wire
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ReadyToExecute {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub trace_id: String,
    /// Carry forward context for post-processing (JMESPath, merge keys, etc.)
    pub context: ExecutionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub provider_name: String,
    pub model_name: String,
    pub node_id: Uuid,
    pub result_key: Option<String>,
    pub output_transform: Option<String>,
    pub input_json: Value, // Original input for merging
    pub start_time: u64,   // For telemetry
}

/// Stage 2 -> Stage 3: Raw response received
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub status: u16,
    pub raw_body: String,
    pub trace_id: String,
    pub context: ExecutionContext,
}
