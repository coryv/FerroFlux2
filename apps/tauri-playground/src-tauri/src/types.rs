use flow_canvas::model::{Node, NodeData, NodeId, PortId, WireStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaygroundNodeData {
    pub name: String,
}

impl NodeData for PlaygroundNodeData {
    fn node_type(&self) -> String {
        "PlaygroundNode".to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableNode {
    pub id: NodeId,
    pub uuid: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
    pub data: PlaygroundNodeData,
}

#[derive(Serialize, Deserialize)]
pub struct SerializableEdge {
    pub id: String,
    pub from: PortId,
    pub to: PortId,
    pub style: WireStyle,
    pub path: Vec<(f32, f32)>,
    pub bezier_control_points: Option<((f32, f32), (f32, f32))>,
}

#[derive(Serialize)]
pub struct SerializableGraph {
    pub nodes: HashMap<NodeId, SerializableNode>,
    pub edges: HashMap<String, SerializableEdge>,
    pub draw_order: Vec<NodeId>,
}

#[derive(Serialize, Deserialize)]
pub struct ClipboardData {
    pub nodes: Vec<Node<PlaygroundNodeData>>,
    pub edges: Vec<SerializableEdge>,
}
