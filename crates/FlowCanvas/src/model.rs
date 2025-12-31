//! # Core Data Models
//!
//! This module defines the fundamental data model for the graph.
//! It uses `SlotMap` for efficient, safe, and stable entity storage without pointers.
//!
//! The graph is generic over `T: NodeData` to allow consumers to embed their own payload.

use glam::Vec2;
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use slotmap::new_key_type;
use std::collections::HashMap;

/// Trait that user data must implement to be stored in the graph.
pub trait NodeData: Clone + std::fmt::Debug {
    /// Returns the registry ID (type string) for this node, used by the engine.
    fn node_type(&self) -> String;
}

// Default implementation for String (commonly used in tests/simple examples)
impl NodeData for String {
    fn node_type(&self) -> String {
        // If the data is just a string, we assume it *is* the type, or "Default".
        // For simple tests using "Node A", "Node B", let's return "Default"
        // unless it looks like a type ID.
        // NOTE: For now, let's just return "Default" to be safe for existing tests
        // that use arbitrary strings like "Node A".
        // Or better: let's return "Default" if it contains spaces, otherwise proper casing?
        // Let's stick to "Default" for simple strings for now to match behavior,
        // OR return self.clone() if we want dynamic string typing.
        // given the user constraint: "return a String ... to ensure FlowCanvas remains decoupled"
        "Default".to_string()
    }
}

// Implementation for Unit type (if needed)
impl NodeData for () {
    fn node_type(&self) -> String {
        "Default".to_string()
    }
}

new_key_type! {
    /// Unique identifier for a Node.
    pub struct NodeId;
    /// Unique identifier for a Port.
    pub struct PortId;
    /// Unique identifier for a Connection.
    pub struct ConnectionId;
}

/// Bitflags representing various boolean states of a Node.
use bitflags::bitflags;

bitflags! {
    /// Bitflags representing various boolean states of a Node.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct NodeFlags: u8 {
        /// The node cannot be moved or deleted.
        const LOCKED = 1 << 0;
        /// The node is not rendered.
        const HIDDEN = 1 << 1;
        /// The node is currently selected by the user.
        const SELECTED = 1 << 2;
    }
}

// Manual Serialize/Deserialize implementation for bitlags to be friendly
impl Serialize for NodeFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

impl<'de> Deserialize<'de> for NodeFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Ok(Self::from_bits_truncate(bits))
    }
}

pub use uuid::Uuid;

/// A Node in the graph.
///
/// Nodes are the primary entities. They have a position, size, and
/// a list of input/output ports. They also carry user-defined `data`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node<T> {
    /// Self-reference ID.
    pub id: NodeId,
    /// Stable UUID for persistence.
    pub uuid: Uuid,
    /// World-space position of the top-left corner.
    pub position: Vec2,
    /// Size of the node layout.
    pub size: Vec2,
    /// List of input port IDs.
    pub inputs: Vec<PortId>,
    /// List of output port IDs.
    pub outputs: Vec<PortId>,
    /// User-defined payload.
    pub data: T,
    /// State flags.
    pub flags: NodeFlags,
    /// Optional visual style override.
    pub style: Option<crate::config::NodeStyle>,
}

/// A Port on a Node.
///
/// Ports are the anchors for Connections.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    /// Self-reference ID.
    pub id: PortId,
    /// ID of the Node this port belongs to.
    pub node: NodeId,
    // Add other relevant port info if needed, e.g., name, type
}

/// Visual style of the connection wire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WireStyle {
    /// A smooth cubic Bezier curve (standard).
    Cubic,
    /// A straight line.
    Linear,
    /// An orthogonal (manhattan) path (L-shaped).
    Orthogonal,
}

/// A Connection between two Ports.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    /// Source Port ID.
    pub from: PortId,
    /// Target Port ID.
    pub to: PortId,
    /// Visual style of the wire (shape).
    pub style: WireStyle,
    /// Optional visual style override (color/width).
    pub visual_style: Option<crate::config::EdgeStyle>,
}

/// The entire state of the Graph.
///
/// This struct holds all entities (Nodes, Ports, Connections, Groups) in flat Arenas (`SlotMap`).
/// It is responsible for data storage, but not for rendering or interaction logic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphState<T> {
    /// Arena for Nodes.
    pub nodes: SlotMap<NodeId, Node<T>>,
    /// Arena for Ports.
    pub ports: SlotMap<PortId, Port>,
    /// Arena for Connections.
    pub connections: SlotMap<ConnectionId, Connection>,
    /// Draw order cache.
    /// Lower index = Background/Bottom.
    /// Higher index = Foreground/Top.
    pub draw_order: Vec<NodeId>,
    /// Index for O(1) UUID to NodeId lookup.
    #[serde(default, skip)]
    pub uuid_index: HashMap<Uuid, NodeId>,
}

impl<T> Default for GraphState<T> {
    fn default() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            ports: SlotMap::with_key(),
            connections: SlotMap::with_key(),
            draw_order: Vec::new(),
            uuid_index: HashMap::new(),
        }
    }
}

impl<T: NodeData> GraphState<T> {
    /// Helper to find the world position of a port.
    pub fn find_port_position(&self, port_id: PortId) -> Option<Vec2> {
        let port = self.ports.get(port_id)?;
        let node = self.nodes.get(port.node)?;

        if let Some(idx) = node.inputs.iter().position(|&id| id == port_id) {
            let spacing = node.size.y / (node.inputs.len() as f32 + 1.0);
            let y = node.position.y + spacing * (idx as f32 + 1.0);
            let x = node.position.x;
            return Some(Vec2::new(x, y));
        }

        if let Some(idx) = node.outputs.iter().position(|&id| id == port_id) {
            let spacing = node.size.y / (node.outputs.len() as f32 + 1.0);
            let y = node.position.y + spacing * (idx as f32 + 1.0);
            let x = node.position.x + node.size.x;
            return Some(Vec2::new(x, y));
        }

        None
    }

    /// Inserts a node and updates the UUID index.
    pub fn insert_node(&mut self, mut node: Node<T>) -> NodeId {
        let id = self.nodes.insert_with_key(|key| {
            node.id = key;
            node
        });
        // We get the node back if we clone or just re-access
        let uuid = self.nodes[id].uuid;
        self.uuid_index.insert(uuid, id);
        id
    }

    /// Removes a node and updates the UUID index.
    pub fn remove_node(&mut self, id: NodeId) -> Option<Node<T>> {
        if let Some(node) = self.nodes.remove(id) {
            self.uuid_index.remove(&node.uuid);
            Some(node)
        } else {
            None
        }
    }
}
