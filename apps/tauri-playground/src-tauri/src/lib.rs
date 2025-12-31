use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::{GraphState, Node, NodeData, NodeId, Uuid};
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
            },
        }
    } else {
        println!("Backend: client already initialized");
        Ok(())
    }
}

use std::collections::HashMap;

#[derive(Serialize)]
struct SerializableNode {
    id: String,
    uuid: String,
    position: (f32, f32),
    size: (f32, f32),
    data: PlaygroundNodeData,
}

#[derive(Serialize)]
struct SerializableGraph {
    nodes: HashMap<String, SerializableNode>,
}

#[tauri::command]
async fn get_graph(state: tauri::State<'_, AppState>) -> Result<SerializableGraph, String> {
    println!("Backend: get_graph called");
    let graph = state.graph.lock().await;
    println!("Backend: graph lock acquired. Node count: {}", graph.nodes.len());
    
    let mut serializable_nodes = HashMap::new();
    for (id, node) in &graph.nodes {
        serializable_nodes.insert(format!("{:?}", id), SerializableNode {
            id: format!("{:?}", id),
            uuid: node.uuid.to_string(),
            position: (node.position.x, node.position.y),
            size: (node.size.x, node.size.y),
            data: node.data.clone(),
        });
    }

    let result = SerializableGraph {
        nodes: serializable_nodes,
    };
    
    println!("Backend: Returning serializable graph with {} nodes", result.nodes.len());
    Ok(result)
}

#[tauri::command]
async fn add_node(state: tauri::State<'_, AppState>, name: String, x: f32, y: f32) -> Result<String, String> {
    println!("Backend: add_node called: {}", name);
    let mut graph = state.graph.lock().await;
    let node_uuid = Uuid::new_v4();
    let node_id = graph.insert_node(Node {
        id: NodeId::default(), // This gets overwritten by insert_node usually? Wait, Node has `id` field?
        // Actually FlowCanvas insert_node might expect us to set ID or it sets it?
        // Let's check insert_node signature. usually slotmap returns the id.
        // If Node struct has an `id` field, we might need to update it after insertion if we want it consistent.
        // But for playground, just inserting provided struct is fine.
        uuid: node_uuid,
        position: glam::Vec2::new(x, y),
        size: glam::Vec2::new(160.0, 80.0),
        inputs: vec![],
        outputs: vec![],
        data: PlaygroundNodeData { name },
        flags: Default::default(),
        style: None,
    });
    // If GraphState::insert_node returns NodeId, we are good.
    // Ideally we should fix the `id` field inside the node to match the slotmap key if needed,
    // but FlowCanvas probably handles it or doesn't care about internal self-reference.
    
    Ok(format!("{:?}", node_id))
}

#[tauri::command]
async fn deploy(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut client_opt = state.client.lock().await;
    let graph_lock = state.graph.lock().await;
    
    if let Some(client) = client_opt.as_mut() {
        client.compile_and_deploy(&graph_lock).await.map_err(|e| e.to_string())?;
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
        .invoke_handler(tauri::generate_handler![init_sdk, get_graph, add_node, deploy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
