use flow_canvas::model::{GraphState, Node, NodeFlags, Port, PortId, Uuid, WireStyle};
use flow_canvas::persistence::SavedGraph;
use glam::Vec2;

#[test]
fn test_roundtrip_persistence() {
    // 1. Create a Graph with 2 Nodes and a Connection
    let mut graph: GraphState<String> = GraphState::default();

    // Node A
    let node_a_uuid = Uuid::new_v4();
    let node_a = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: node_a_uuid,
        position: Vec2::new(0.0, 0.0),
        size: Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: "Node A".to_string(),
        flags: NodeFlags::default(),
        style: None,
    });
    // Create Output Port
    let port_out = graph.ports.insert(Port {
        id: PortId::default(),
        node: node_a,
    });
    graph.nodes[node_a].outputs.push(port_out);

    // Node B
    let node_b_uuid = Uuid::new_v4();
    let node_b = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: node_b_uuid,
        position: Vec2::new(200.0, 0.0),
        size: Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: "Node B".to_string(),
        flags: NodeFlags::default(),
        style: None,
    });
    // Create Input Port
    let port_in = graph.ports.insert(Port {
        id: PortId::default(),
        node: node_b,
    });
    graph.nodes[node_b].inputs.push(port_in);

    // Connection
    graph.connections.insert(flow_canvas::model::Connection {
        from: port_out,
        to: port_in,
        style: WireStyle::Cubic,
        visual_style: None,
    });

    // 2. Save
    let saved: SavedGraph<String> = graph.save();

    // Verify Saved State content
    assert_eq!(saved.nodes.len(), 2);
    assert_eq!(saved.connections.len(), 1);

    // Check connection UUIDs
    let conn = &saved.connections[0];
    assert_eq!(conn.from_node, node_a_uuid);
    assert_eq!(conn.to_node, node_b_uuid);
    assert_eq!(conn.from_port_index, 0); // First output
    assert_eq!(conn.to_port_index, 0); // First input

    // 3. Load into NEW Graph
    let mut new_graph: GraphState<String> = GraphState::default();
    new_graph.load(saved);

    // 4. Verify Loaded Graph
    assert_eq!(new_graph.nodes.len(), 2);
    assert_eq!(new_graph.connections.len(), 1);

    // Find nodes by UUID (inefficient linear search for validation)
    let new_node_a = new_graph
        .nodes
        .values()
        .find(|n| n.uuid == node_a_uuid)
        .expect("Node A missing");
    let new_node_b = new_graph
        .nodes
        .values()
        .find(|n| n.uuid == node_b_uuid)
        .expect("Node B missing");

    assert_eq!(new_node_a.data, "Node A");
    assert_eq!(new_node_b.data, "Node B");
    assert_eq!(new_node_a.outputs.len(), 1);
    assert_eq!(new_node_b.inputs.len(), 1);

    // Verify Connection
    let new_conn = new_graph
        .connections
        .values()
        .next()
        .expect("Connection missing");
    let new_from_port = new_graph
        .ports
        .get(new_conn.from)
        .expect("From port missing");
    let new_to_port = new_graph.ports.get(new_conn.to).expect("To port missing");

    assert_eq!(new_from_port.node, new_node_a.id); // Not stable ID, but correct relationship
    assert_eq!(new_to_port.node, new_node_b.id);
}
