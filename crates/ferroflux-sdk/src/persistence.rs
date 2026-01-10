use ferroflux_core::components::core::{Inbox, Outbox};
use ferroflux_core::store::blob::BlobSnapshot;
use flow_canvas::model::GraphState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the execution state of a specific node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRuntimeState {
    pub inbox: Inbox,
    pub outbox: Outbox,
}

/// Represents the complete runtime state of the engine (excluding graph definition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub node_states: HashMap<Uuid, NodeRuntimeState>,
    pub blobs: Vec<BlobSnapshot>,
}

/// A complete save file containing both the graph definition and the runtime state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile<T> {
    pub graph: GraphState<T>,
    pub runtime: RuntimeSnapshot,
}
