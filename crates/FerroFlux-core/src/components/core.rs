use super::super::store::SecureTicket; // crate::store::SecureTicket
use crate::domain::TenantId;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Basic identity for a node in the graph.
///
/// Contains metadata like name, type, and workflow association.
/// Required for almost every node entity.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique identifier for this node instance.
    pub id: Uuid,
    /// Human-readable label (e.g., "Customer Agent").
    pub name: String,
    /// The string identifier for the node type (e.g., "Agent", "Http").
    #[serde(skip_deserializing, default)]
    pub node_type: String,
    /// The ID of the workflow this node belongs to.
    /// Optional to support legacy nodes (or "global" nodes in the future).
    #[serde(default)]
    pub workflow_id: Option<String>,
    /// The tenant this node belongs to.
    #[serde(default)]
    pub tenant_id: Option<TenantId>,
}

/// A component that acts as a "mock" or "override" for a node's output.
///
/// If this component is present, the worker system should skip normal execution
/// and instead push the contained `SecureTicket` directly to the `Outbox`.
/// Useful for testing, debugging, or forcing a specific path.
#[derive(Component, Debug, Clone)]
pub struct PinnedOutput(pub SecureTicket);

/// A directed edge connecting two entities in the dataflow graph.
///
/// Represents the flow of data. The `System` iterates over these to move
/// tickets from a `Outbox` to an `Inbox`.
#[derive(Component, Debug, Clone)]
pub struct Edge {
    /// The entity ID of the source node.
    pub source: Entity,
    /// The entity ID of the target node.
    pub target: Entity,
}

/// Holds incoming data packets waiting to be processed.
#[derive(Component, Debug, Clone, Default)]
pub struct Inbox {
    pub queue: VecDeque<SecureTicket>,
}

/// Holds outgoing data packets waiting to be routed to the next node.
#[derive(Component, Debug, Clone, Default)]
pub struct Outbox {
    pub queue: VecDeque<SecureTicket>,
}

/// Helper tag to label edges for logic branching (e.g., "true", "false", "default").
///
/// Logic nodes (like Switch) use this to determine which edge(s) to traverse
/// based on their evaluation result.
#[derive(Component, Debug, Clone)]
pub struct EdgeLabel(pub String);
