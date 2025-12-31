use crate::engine::EngineCommand;
use crate::state::AppState;
use crate::types::{
    ClipboardData, PlaygroundNodeData, SerializableEdge, SerializableGraph, SerializableNode,
};
use flow_canvas::model::{ConnectionId, Node, NodeId, PortId, Uuid, WireStyle};
use std::collections::HashMap;
use tokio::sync::oneshot;

#[tauri::command]
pub async fn init_sdk(state: tauri::State<'_, AppState>) -> Result<(), String> {
    println!("Backend: init_sdk requested");
    let (tx, rx) = oneshot::channel();
    state
        .engine_tx
        .send(EngineCommand::Init(tx))
        .await
        .map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn log_js(msg: String) {
    println!("JS: {}", msg);
}

#[tauri::command]
pub async fn get_graph(state: tauri::State<'_, AppState>) -> Result<SerializableGraph, String> {
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

        let start_pos = graph
            .find_port_position(conn.from)
            .unwrap_or(glam::Vec2::ZERO);
        let end_pos = graph
            .find_port_position(conn.to)
            .unwrap_or(glam::Vec2::ZERO);

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

    Ok(SerializableGraph {
        nodes: serializable_nodes,
        edges: serializable_edges,
        draw_order: graph.draw_order.clone(),
    })
}

#[tauri::command]
pub async fn bring_to_front(state: tauri::State<'_, AppState>, id: NodeId) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    graph.draw_order.retain(|&node_id| node_id != id);
    graph.draw_order.push(id);
    Ok(())
}

#[tauri::command]
pub async fn undo(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;
    if history.undo(&mut graph) {
        Ok(())
    } else {
        Err("Nothing to undo".to_string())
    }
}

#[tauri::command]
pub async fn redo(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;
    if history.redo(&mut graph) {
        Ok(())
    } else {
        Err("Nothing to redo".to_string())
    }
}

#[tauri::command]
pub async fn set_connection_wire_style(
    state: tauri::State<'_, AppState>,
    id: String,
    style: WireStyle,
) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;

    let mut found_id = None;
    for (conn_id, _) in &graph.connections {
        if format!("{:?}", conn_id) == id {
            found_id = Some(conn_id);
            break;
        }
    }

    if let Some(conn_id) = found_id {
        history.commit(&graph);
        graph.set_connection_style(conn_id, style);
        Ok(())
    } else {
        Err("Connection not found".to_string())
    }
}

#[tauri::command]
pub async fn set_all_connection_wire_styles(
    state: tauri::State<'_, AppState>,
    style: WireStyle,
) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;
    let mut default_style = state.default_wire_style.lock().await;

    history.commit(&graph);
    *default_style = style.clone();

    for (_, conn) in &mut graph.connections {
        conn.style = style.clone();
    }

    Ok(())
}

#[tauri::command]
pub async fn add_node(
    state: tauri::State<'_, AppState>,
    name: String,
    x: f32,
    y: f32,
) -> Result<String, String> {
    println!("Backend: add_node called: {}", name);
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;

    history.commit(&graph);

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

    graph.draw_order.push(node_id);
    graph.add_port(node_id, true);
    graph.add_port(node_id, false);

    Ok(format!("{:?}", node_id))
}

#[tauri::command]
pub async fn add_edge(
    state: tauri::State<'_, AppState>,
    from: PortId,
    to: PortId,
) -> Result<String, String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;
    let default_style = state.default_wire_style.lock().await;

    history.commit(&graph);

    let conn_id = graph.connect_with_style(from, to, default_style.clone());
    Ok(format!("{:?}", conn_id))
}

#[tauri::command]
pub async fn update_node_position(
    state: tauri::State<'_, AppState>,
    id: NodeId,
    x: f32,
    y: f32,
    commit: bool,
) -> Result<(), String> {
    let mut graph = state.graph.lock().await;

    if commit {
        let mut history = state.history.lock().await;
        history.commit(&graph);
    }

    if let Some(node) = graph.nodes.get_mut(id) {
        node.position = glam::Vec2::new(x, y);
        Ok(())
    } else {
        Err(format!("Node {:?} not found", id))
    }
}

#[tauri::command]
pub async fn delete_items(
    state: tauri::State<'_, AppState>,
    nodes: Vec<NodeId>,
    edges: Vec<String>,
) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;

    history.commit(&graph);

    let mut edge_ids_to_remove = Vec::new();
    for (conn_id, _) in &graph.connections {
        if edges.contains(&format!("{:?}", conn_id)) {
            edge_ids_to_remove.push(conn_id);
        }
    }
    for id in edge_ids_to_remove {
        graph.connections.remove(id);
    }

    for node_id in nodes {
        let mut ports_to_remove = Vec::new();
        if let Some(node) = graph.nodes.get(node_id) {
            ports_to_remove.extend(&node.inputs);
            ports_to_remove.extend(&node.outputs);
        }

        let edges_to_remove: Vec<ConnectionId> = graph
            .connections
            .iter()
            .filter(|(_, conn)| {
                ports_to_remove.contains(&conn.from) || ports_to_remove.contains(&conn.to)
            })
            .map(|(id, _)| id)
            .collect();

        for id in edges_to_remove {
            graph.connections.remove(id);
        }

        for pid in ports_to_remove {
            graph.ports.remove(pid);
        }

        graph.nodes.remove(node_id);
        graph.draw_order.retain(|&id| id != node_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn copy_items(
    state: tauri::State<'_, AppState>,
    nodes: Vec<NodeId>,
) -> Result<String, String> {
    let graph = state.graph.lock().await;

    let mut clipboard_nodes = Vec::new();
    let mut port_to_node = HashMap::new();

    for id in &nodes {
        if let Some(node) = graph.nodes.get(*id) {
            clipboard_nodes.push(node.clone());
            for port_id in node.inputs.iter().chain(node.outputs.iter()) {
                port_to_node.insert(*port_id, *id);
            }
        }
    }

    let mut clipboard_edges = Vec::new();
    for (conn_id, conn) in &graph.connections {
        if port_to_node.contains_key(&conn.from) && port_to_node.contains_key(&conn.to) {
            clipboard_edges.push(SerializableEdge {
                id: format!("{:?}", conn_id),
                from: conn.from,
                to: conn.to,
                style: conn.style.clone(),
                path: vec![],
                bezier_control_points: None,
            });
        }
    }

    serde_json::to_string(&ClipboardData {
        nodes: clipboard_nodes,
        edges: clipboard_edges,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn paste_items(
    state: tauri::State<'_, AppState>,
    json: String,
    x: f32,
    y: f32,
) -> Result<(), String> {
    let mut graph = state.graph.lock().await;
    let mut history = state.history.lock().await;

    let data: ClipboardData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if data.nodes.is_empty() {
        return Ok(());
    }

    history.commit(&graph);
    let mut avg_x = 0.0;
    let mut avg_y = 0.0;
    for n in &data.nodes {
        avg_x += n.position.x;
        avg_y += n.position.y;
    }
    avg_x /= data.nodes.len() as f32;
    avg_y /= data.nodes.len() as f32;

    let offset_x = x - avg_x;
    let offset_y = y - avg_y;

    let mut port_map = HashMap::new();

    for mut node in data.nodes {
        let old_inputs = node.inputs.clone();
        let old_outputs = node.outputs.clone();

        node.id = NodeId::default();
        node.uuid = Uuid::new_v4();
        node.position.x += offset_x;
        node.position.y += offset_y;
        node.inputs.clear();
        node.outputs.clear();

        let new_node_id = graph.insert_node(node);
        graph.draw_order.push(new_node_id);

        for old_id in old_inputs {
            let new_id = graph.add_port(new_node_id, true);
            port_map.insert(old_id, new_id);
        }
        for old_id in old_outputs {
            let new_id = graph.add_port(new_node_id, false);
            port_map.insert(old_id, new_id);
        }
    }

    for edge in data.edges {
        if let (Some(&new_from), Some(&new_to)) = (port_map.get(&edge.from), port_map.get(&edge.to))
        {
            graph.connect_with_style(new_from, new_to, edge.style);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn deploy(state: tauri::State<'_, AppState>) -> Result<(), String> {
    println!("Backend: deploy requested");
    let graph = state.graph.lock().await.clone();
    let (tx, rx) = oneshot::channel();
    state
        .engine_tx
        .send(EngineCommand::Deploy(graph, tx))
        .await
        .map_err(|e| e.to_string())?;
    rx.await.map_err(|e| e.to_string())?
}
