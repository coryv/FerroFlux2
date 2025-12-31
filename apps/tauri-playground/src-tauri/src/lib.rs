use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::{GraphState, Node, NodeData, NodeId, PortId, Uuid, WireStyle};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlaygroundNodeData {
    name: String,
}

impl NodeData for PlaygroundNodeData {
    fn node_type(&self) -> String {
        "PlaygroundNode".to_string()
    }
}

struct AppState {
    client: Arc<Mutex<Option<FerroFluxClient<PlaygroundNodeData>>>>,
    graph: Arc<Mutex<GraphState<PlaygroundNodeData>>>,
}

#[tauri::command]
async fn init_sdk(state: tauri::State<'_, AppState>) -> Result<(), String> {
    println!("Backend: init_sdk called");
    let mut client_lock = state.client.lock().await;
    if client_lock.is_none() {
        println!("Backend: initializing new client...");
        match FerroFluxClient::init().await {
            Ok(client) => {
                println!("Backend: client initialized successfully");
                *client_lock = Some(client);
                Ok(())
            }
            Err(e) => {
                println!("Backend: client init failed: {:?}", e);
                Err(format!("Failed to init SDK: {:?}", e))
            }
        }
    } else {
        println!("Backend: client already initialized");
        Ok(())
    }
}

use std::collections::HashMap;

#[derive(Serialize)]
struct SerializableNode {
    id: NodeId,
    uuid: String,
    position: (f32, f32),
    size: (f32, f32),
    inputs: Vec<PortId>,
    outputs: Vec<PortId>,
    data: PlaygroundNodeData,
}

#[derive(Serialize)]
struct SerializableEdge {
    id: String,
    from: PortId,
    to: PortId,
    style: WireStyle,
    path: Vec<(f32, f32)>,
    bezier_control_points: Option<((f32, f32), (f32, f32))>,
}

#[derive(Serialize)]
struct SerializableGraph {
    nodes: HashMap<NodeId, SerializableNode>,
    edges: HashMap<String, SerializableEdge>,
    draw_order: Vec<NodeId>,
}

#[tauri::command]
async fn get_graph(state: tauri::State<'_, AppState>) -> Result<SerializableGraph, String> {
    println!("Backend: get_graph called");
    let graph = state.graph.lock().await;

    let mut serializable_nodes = HashMap::new();
    for (id, node) in &graph.nodes {
        serializable_nodes.insert(
            id,
            SerializableNode {
                id,
                uuid: node.uuid.to_string(),
                position: (node.position.x, node.position.y),
                size: (node.size.x, node.size.y),
                inputs: node.inputs.clone(),
                outputs: node.outputs.clone(),
                data: node.data.clone(),
            },
        );
    }

    let mut serializable_edges = HashMap::new();
    for (id, conn) in &graph.connections {
        let edge_id = format!("{:?}", id);

        // Calculate path points using FlowCanvas math
        let start_pos = graph
            .find_port_position(conn.from)
            .unwrap_or(glam::Vec2::ZERO);
        let end_pos = graph
            .find_port_position(conn.to)
            .unwrap_or(glam::Vec2::ZERO);

        // Identify which nodes to ignore as obstacles (the nodes this edge connects)

        let obstacles: Vec<flow_canvas::math::Rect> = graph
            .nodes
            .iter()
            .map(|(_, node)| flow_canvas::math::Rect::new(node.position, node.size))
            .collect();

        let mut bezier_cp = None;
        let path = match conn.style {
            WireStyle::Cubic => {
                let (cp1, cp2) = flow_canvas::math::calculate_bezier_points(start_pos, end_pos);
                bezier_cp = Some(((cp1.x, cp1.y), (cp2.x, cp2.y)));
                vec![(start_pos.x, start_pos.y), (end_pos.x, end_pos.y)]
            }
            WireStyle::Linear => flow_canvas::math::calculate_linear_points(start_pos, end_pos)
                .into_iter()
                .map(|v| (v.x, v.y))
                .collect(),
            WireStyle::Orthogonal => {
                flow_canvas::math::calculate_smart_orthogonal(start_pos, end_pos, &obstacles, 20.0)
                    .into_iter()
                    .map(|v| (v.x, v.y))
                    .collect()
            }
        };

        serializable_edges.insert(
            edge_id.clone(),
            SerializableEdge {
                id: edge_id,
                from: conn.from,
                to: conn.to,
                style: conn.style.clone(),
                path,
                bezier_control_points: bezier_cp,
            },
        );
    }

    let result = SerializableGraph {
        nodes: serializable_nodes,
        edges: serializable_edges,
        draw_order: graph.draw_order.clone(),
    };

    Ok(result)
}

#[tauri::command]
async fn bring_to_front(state: tauri::State<'_, AppState>, id: NodeId) -> Result<(), String> {
    println!("Backend: bring_to_front called for node: {:?}", id);
    let mut graph = state.graph.lock().await;

    // Move the node to the end of the draw order
    graph.draw_order.retain(|&node_id| node_id != id);
    graph.draw_order.push(id);

    Ok(())
}

#[tauri::command]
async fn set_connection_wire_style(
    state: tauri::State<'_, AppState>,
    id: String,
    style: WireStyle,
) -> Result<(), String> {
    println!(
        "Backend: set_connection_wire_style called for: {} to {:?}",
        id, style
    );
    let mut graph = state.graph.lock().await;

    // Parse the ConnectionId from the stringified SlotMap key
    // This is a bit tricky with SlotMap keys as strings,
    // but we can assume the format matches what we send to frontend.
    // However, since we don't have a direct parser for SlotMap string keys easily available,
    // we'll iterate and find it.
    let mut found_id = None;
    for (conn_id, _) in &graph.connections {
        if format!("{:?}", conn_id) == id {
            found_id = Some(conn_id);
            break;
        }
    }

    if let Some(conn_id) = found_id {
        graph.set_connection_style(conn_id, style);
        Ok(())
    } else {
        Err("Connection not found".to_string())
    }
}

#[tauri::command]
async fn add_node(
    state: tauri::State<'_, AppState>,
    name: String,
    x: f32,
    y: f32,
) -> Result<String, String> {
    println!("Backend: add_node called: {}", name);
    let mut graph = state.graph.lock().await;

    let node_uuid = Uuid::new_v4();
    let node_id = graph.insert_node(Node {
        id: NodeId::default(),
        uuid: node_uuid,
        position: glam::Vec2::new(x, y),
        size: glam::Vec2::new(160.0, 80.0),
        inputs: vec![],
        outputs: vec![],
        data: PlaygroundNodeData { name },
        flags: Default::default(),
        style: None,
    });

    // Update draw order for new node
    graph.draw_order.push(node_id);

    // Add default ports
    graph.add_port(node_id, true); // Input
    graph.add_port(node_id, false); // Output

    Ok(format!("{:?}", node_id))
}

#[tauri::command]
async fn add_edge(
    state: tauri::State<'_, AppState>,
    from: PortId,
    to: PortId,
) -> Result<String, String> {
    println!("Backend: add_edge called: {:?} -> {:?}", from, to);
    let mut graph = state.graph.lock().await;

    let conn_id = graph.connect(from, to);
    Ok(format!("{:?}", conn_id))
}

#[tauri::command]
async fn update_node_position(
    state: tauri::State<'_, AppState>,
    id: NodeId,
    x: f32,
    y: f32,
) -> Result<(), String> {
    println!(
        "Backend: update_node_position called for node: {:?} to ({}, {})",
        id, x, y
    );
    let mut graph = state.graph.lock().await;

    if let Some(node) = graph.nodes.get_mut(id) {
        node.position = glam::Vec2::new(x, y);
        Ok(())
    } else {
        Err(format!("Node {:?} not found", id))
    }
}

#[tauri::command]
async fn deploy(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut client_opt = state.client.lock().await;
    let graph_lock = state.graph.lock().await;

    if let Some(client) = client_opt.as_mut() {
        client
            .compile_and_deploy(&graph_lock)
            .await
            .map_err(|e| e.to_string())?;
        // Run a tick just to kickstart
        client.tick().await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("SDK not initialized".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            client: Arc::new(Mutex::new(None)),
            graph: Arc::new(Mutex::new(GraphState::default())),
        })
        .invoke_handler(tauri::generate_handler![
            init_sdk,
            get_graph,
            add_node,
            add_edge,
            bring_to_front,
            set_connection_wire_style,
            update_node_position,
            deploy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
