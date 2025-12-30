use crate::model::{self, Connection, GraphState, Node, NodeData, Port, PortId, WireStyle};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A serializable representation of a Connection.
///
/// Instead of transient `PortId`s, it uses Stable UUIDs for nodes and indices for ports.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedConnection {
    pub from_node: Uuid,
    pub from_port_index: usize,
    pub to_node: Uuid,
    pub to_port_index: usize,
    pub style: WireStyle,
    pub visual_style: Option<crate::config::EdgeStyle>,
}

/// A serializable representation of a Node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedNode<T> {
    pub uuid: Uuid,
    pub position: Vec2,
    pub size: Vec2,
    pub data: T,
    pub flags: model::NodeFlags,
    pub style: Option<crate::config::NodeStyle>,
    /// We save the number of ports to recreate them.
    /// If ports had distinct data (names, types), we would save a `SavedPort` struct here or in a list.
    /// For now, ports are structural.
    pub input_count: usize,
    pub output_count: usize,
}

/// A serializable snapshot of the Graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedGraph<T> {
    pub nodes: Vec<SavedNode<T>>,
    pub connections: Vec<SavedConnection>,
}

impl<T: NodeData + Serialize + for<'de> Deserialize<'de>> GraphState<T> {
    /// Serializes the graph state into a `SavedGraph` payload.
    pub fn save(&self) -> SavedGraph<T> {
        let mut saved_nodes = Vec::new();
        let mut node_id_to_uuid = HashMap::new();

        for (id, node) in &self.nodes {
            saved_nodes.push(SavedNode {
                uuid: node.uuid,
                position: node.position,
                size: node.size,
                data: node.data.clone(),
                flags: node.flags,
                style: node.style.clone(),
                input_count: node.inputs.len(),
                output_count: node.outputs.len(),
            });
            node_id_to_uuid.insert(id, node.uuid);
        }

        let mut saved_connections = Vec::new();
        for (_id, conn) in &self.connections {
            // Resolve Ports to Nodes
            let from_port = self
                .ports
                .get(conn.from)
                .expect("Connection with invalid From Port");
            let to_port = self
                .ports
                .get(conn.to)
                .expect("Connection with invalid To Port");

            let from_node = self
                .nodes
                .get(from_port.node)
                .expect("Port with invalid Node");
            let to_node = self
                .nodes
                .get(to_port.node)
                .expect("Port with invalid Node");

            // Find index of port in node's list
            let from_idx = from_node
                .outputs
                .iter()
                .position(|&p| p == conn.from)
                .unwrap_or(0);
            let to_idx = to_node
                .inputs
                .iter()
                .position(|&p| p == conn.to)
                .unwrap_or(0);

            saved_connections.push(SavedConnection {
                from_node: from_node.uuid,
                from_port_index: from_idx,
                to_node: to_node.uuid,
                to_port_index: to_idx,
                style: conn.style.clone(),
                visual_style: conn.visual_style.clone(),
            });
        }

        SavedGraph {
            nodes: saved_nodes,
            connections: saved_connections,
        }
    }

    /// Loads a `SavedGraph` payload, REPLACING the current state.
    pub fn load(&mut self, saved: SavedGraph<T>) {
        // Clear current state using SlotMap clear (retain keys? No, completely new)
        self.nodes.clear();
        self.ports.clear();
        self.connections.clear();
        self.draw_order.clear();

        let mut uuid_to_new_id = HashMap::new();

        // 1. Restore Nodes
        for saved_node in saved.nodes {
            let node_id = self.nodes.insert_with_key(|key| {
                // We need to create Ports first to put in struct,
                // BUT ports need NodeId. Circular dep with SlotMap insert?
                // SlotMap allows `insert_with_key`.
                Node {
                    id: key,
                    uuid: saved_node.uuid,
                    position: saved_node.position,
                    size: saved_node.size,
                    inputs: Vec::new(), // Will fill momentarily
                    outputs: Vec::new(),
                    data: saved_node.data,
                    flags: saved_node.flags,
                    style: saved_node.style,
                }
            });

            // Create Ports
            let mut inputs = Vec::new();
            for _ in 0..saved_node.input_count {
                inputs.push(self.ports.insert(Port {
                    id: PortId::default(), // overwritten
                    node: node_id,
                }));
            }
            let mut outputs = Vec::new();
            for _ in 0..saved_node.output_count {
                outputs.push(self.ports.insert(Port {
                    id: PortId::default(), // overwritten
                    node: node_id,
                }));
            }

            // Fixup Port IDs and Node lists
            if let Some(node) = self.nodes.get_mut(node_id) {
                // Fixup port IDs in Port struct is automatic (insert returns key)
                // But we need to put key in port.id if we want it there (model::Port has id field)
                for &pid in &inputs {
                    if let Some(p) = self.ports.get_mut(pid) {
                        p.id = pid;
                    }
                }
                for &pid in &outputs {
                    if let Some(p) = self.ports.get_mut(pid) {
                        p.id = pid;
                    }
                }

                node.inputs = inputs;
                node.outputs = outputs;
            }

            self.draw_order.push(node_id);
            uuid_to_new_id.insert(saved_node.uuid, node_id);
        }

        // 2. Restore Connections
        for saved_conn in saved.connections {
            if let (Some(&from_node_id), Some(&to_node_id)) = (
                uuid_to_new_id.get(&saved_conn.from_node),
                uuid_to_new_id.get(&saved_conn.to_node),
            ) {
                // Find ports
                let from_port = self
                    .nodes
                    .get(from_node_id)
                    .and_then(|n| n.outputs.get(saved_conn.from_port_index).copied());
                let to_port = self
                    .nodes
                    .get(to_node_id)
                    .and_then(|n| n.inputs.get(saved_conn.to_port_index).copied());

                if let (Some(from), Some(to)) = (from_port, to_port) {
                    self.connections.insert(Connection {
                        from,
                        to,
                        style: saved_conn.style,
                        visual_style: saved_conn.visual_style,
                    });
                }
            }
        }
    }
}
