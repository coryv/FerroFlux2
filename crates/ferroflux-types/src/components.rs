use bevy_ecs::prelude::*;
use crate::blob::SecureTicket;
use crate::tenant::TenantId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Basic identity for a node in the graph.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_deserializing, default)]
    pub node_type: String,
    pub workflow_id: String,
    pub tenant_id: TenantId,
}

/// A component that acts as a "mock" or "override" for a node's output.
#[derive(Component, Debug, Clone)]
pub struct PinnedOutput(pub SecureTicket);

/// A directed edge connecting two entities in the dataflow graph.
#[derive(Component, Debug, Clone)]
pub struct Edge {
    pub source: Entity,
    pub source_handle: Option<String>,
    pub target: Entity,
    pub target_handle: Option<String>,
}

/// Holds incoming data packets waiting to be processed.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inbox {
    pub queue: VecDeque<(Option<String>, SecureTicket)>,
}

/// Holds outgoing data packets waiting to be routed to the next node.
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Outbox {
    pub queue: VecDeque<(Option<String>, SecureTicket)>,
}

/// Helper tag to label edges for logic branching (e.g., "true", "false", "default").
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLabel(pub String);
