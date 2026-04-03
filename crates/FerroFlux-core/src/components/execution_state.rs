//! Shared execution state types, re-exported from `ferroflux-types`.
pub use ferroflux_types::data_ref::{ActiveWorkflowState, DataRef};
use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub config: std::collections::HashMap<String, serde_json::Value>,
}
