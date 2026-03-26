use ferroflux_core::nodes::yaml_factory::YamlNodeFactory;
use ferroflux_core::resources::registry::{DefinitionRegistry, NodeRegistry};
use flow_canvas::model::GraphState;
use std::sync::Mutex;
use tauri::Manager;

type Graph = GraphState<String>;

struct AppState {
    graph: Mutex<Graph>,
    registry: Mutex<DefinitionRegistry>,
}

#[derive(serde::Serialize)]
struct Vec2Dto {
    x: f32,
    y: f32,
}

#[derive(serde::Serialize)]
struct PortDto {
    id: String,
    node_id: String,
}

#[derive(serde::Serialize)]
struct NodeDto {
    id: String,
    uuid: String,
    position: Vec2Dto,
    size: Vec2Dto,
    inputs: Vec<String>,
    outputs: Vec<String>,
    data: String,
}

#[derive(serde::Serialize)]
struct ConnectionDto {
    id: String,
    from: String,
    to: String,
}

#[derive(serde::Serialize)]
struct GraphDto {
    nodes: std::collections::HashMap<String, NodeDto>,
    ports: std::collections::HashMap<String, PortDto>,
    connections: Vec<ConnectionDto>,
    draw_order: Vec<String>,
}

#[tauri::command]
fn get_graph(state: tauri::State<AppState>) -> String {
    let graph = state.graph.lock().unwrap();

    // Convert to DTO
    let mut nodes_map = std::collections::HashMap::new();
    for (id, node) in &graph.nodes {
        // We need a stable string representation.
        // The macro uses `self.data().as_ffi().to_string()` for serialize.
        // Let's match that manually to be safe or use the Key's behavior?
        // Let's use the serde behavior:
        let id_str = serde_json::to_string(&id)
            .unwrap()
            .trim_matches('"')
            .to_string();

        let n = NodeDto {
            id: id_str.clone(),
            uuid: node.uuid.to_string(),
            position: Vec2Dto {
                x: node.position.x,
                y: node.position.y,
            },
            size: Vec2Dto {
                x: node.size.x,
                y: node.size.y,
            },
            inputs: node
                .inputs
                .iter()
                .map(|p| {
                    serde_json::to_string(p)
                        .unwrap()
                        .trim_matches('"')
                        .to_string()
                })
                .collect(),
            outputs: node
                .outputs
                .iter()
                .map(|p| {
                    serde_json::to_string(p)
                        .unwrap()
                        .trim_matches('"')
                        .to_string()
                })
                .collect(),
            data: node.data.clone(),
        };
        nodes_map.insert(id_str, n);
    }

    let mut ports_map = std::collections::HashMap::new();
    for (id, port) in &graph.ports {
        let id_str = serde_json::to_string(&id)
            .unwrap()
            .trim_matches('"')
            .to_string();
        let node_id_str = serde_json::to_string(&port.node)
            .unwrap()
            .trim_matches('"')
            .to_string();
        ports_map.insert(
            id_str.clone(),
            PortDto {
                id: id_str,
                node_id: node_id_str,
            },
        );
    }

    let mut connections_vec = Vec::new();
    for (id, conn) in &graph.connections {
        connections_vec.push(ConnectionDto {
            id: serde_json::to_string(&id)
                .unwrap()
                .trim_matches('"')
                .to_string(),
            from: serde_json::to_string(&conn.from)
                .unwrap()
                .trim_matches('"')
                .to_string(),
            to: serde_json::to_string(&conn.to)
                .unwrap()
                .trim_matches('"')
                .to_string(),
        });
    }

    let dto = GraphDto {
        nodes: nodes_map,
        ports: ports_map,
        connections: connections_vec,
        draw_order: graph
            .draw_order
            .iter()
            .map(|id| {
                serde_json::to_string(id)
                    .unwrap()
                    .trim_matches('"')
                    .to_string()
            })
            .collect(),
    };

    serde_json::to_string(&dto).unwrap_or_else(|e| format!("{{ \"error\": \"{}\" }}", e))
}

#[tauri::command]
fn get_node_templates(
    state: tauri::State<AppState>,
) -> Vec<ferroflux_core::traits::node_factory::NodeMetadata> {
    let registry = state.registry.lock().unwrap();
    let mut node_registry = NodeRegistry::new();

    // Register YAML nodes
    for (id, def) in &registry.definitions {
        node_registry.register(id, Box::new(YamlNodeFactory::new(def.clone())));
    }

    node_registry.list_templates()
}

// Add Node command (Mock implementation for now to satisfy interface)
#[tauri::command]
fn add_node(state: tauri::State<AppState>, template_id: String, x: f32, y: f32) -> String {
    let mut graph = state.graph.lock().unwrap();
    // In a real implementation, we would use a factory to create the node.
    // For now, let's just use flow_canvas directly to create a dummy node with the right ID.

    let registry_lock = state.registry.lock().unwrap();

    // 1. Create the node with empty ports first
    let node = flow_canvas::model::Node {
        id: Default::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: glam::Vec2::new(x, y),
        size: glam::Vec2::new(200.0, 200.0), // Default size matches frontend min-w-[200px]
        inputs: vec![],
        outputs: vec![],
        data: template_id.clone(),
        flags: Default::default(),
        style: None,
    };

    // 2. Insert into graph to get ID
    let node_id = graph.insert_node(node);

    // 3. Look up definition to create ports
    if let Some(def) = registry_lock.definitions.get(&template_id) {
        // Create Input Ports
        for _input in &def.interface.inputs {
            graph.add_port(node_id, true);
        }
        // Create Output Ports
        for _output in &def.interface.outputs {
            graph.add_port(node_id, false);
        }
    } else {
        eprintln!(
            "Warning: Template {} not found when creating node",
            template_id
        );
    }

    format!("{:?}", node_id)
}

#[tauri::command]
fn connect_ports(
    state: tauri::State<AppState>,
    from: String,
    to: String,
) -> Result<String, String> {
    let mut graph = state.graph.lock().unwrap();

    // We need to find the PortIds from the string IDs
    // Since we are using string IDs in the frontend that match the serialized debug keys,
    // we need a way to look them up.
    // However, our GraphState is using SlotMap keys which are difficult to reverse look up
    // without iterating.

    // For this prototype, we will iterate the ports map to find the matching keys.
    // In a real app, we might maintain a lookup or use Uuid as the stable key.

    let mut from_port_id = None;
    let mut to_port_id = None;

    let from_clean = from.trim_matches('"');
    let to_clean = to.trim_matches('"');

    for (id, _port) in &graph.ports {
        let _id_str = format!("{:?}", id);
        // The DTO sent trimmed strings, but standard debug format might be different?
        // Let's use the exact same serialization logic as get_graph to match.
        let serialized = serde_json::to_string(&id).map_err(|e| e.to_string())?;
        let clean = serialized.trim_matches('"');

        if clean == from_clean {
            from_port_id = Some(id);
        }
        if clean == to_clean {
            to_port_id = Some(id);
        }

        if from_port_id.is_some() && to_port_id.is_some() {
            break;
        }
    }

    match (from_port_id, to_port_id) {
        (Some(src), Some(dst)) => {
            graph.connect(src, dst);
            Ok("Connected".to_string())
        }
        _ => Err(format!(
            "Could not find ports: from={:?} to={:?}",
            from_port_id, to_port_id
        )),
    }
}

#[tauri::command]
fn update_node_position(
    state: tauri::State<AppState>,
    id: String,
    x: f32,
    y: f32,
) -> Result<String, String> {
    let mut graph = state.graph.lock().unwrap();

    let id_clean = id.trim_matches('"');
    let mut node_key = None;

    for (k, _) in &graph.nodes {
        let serialized = serde_json::to_string(&k).map_err(|e| e.to_string())?;
        if serialized.trim_matches('"') == id_clean {
            node_key = Some(k);
            break;
        }
    }

    if let Some(key) = node_key {
        if let Some(node) = graph.nodes.get_mut(key) {
            node.position = glam::Vec2::new(x, y);
            Ok("Updated".to_string())
        } else {
            Err("Node not found".to_string())
        }
    } else {
        Err("Node ID match not found".to_string())
    }
}

#[tauri::command]
fn delete_node(state: tauri::State<AppState>, id: String) -> Result<String, String> {
    let mut graph = state.graph.lock().unwrap();
    let id_clean = id.trim_matches('"');

    // Find Node Key
    let mut node_key = None;
    for (k, _) in &graph.nodes {
        let serialized = serde_json::to_string(&k).map_err(|e| e.to_string())?;
        if serialized.trim_matches('"') == id_clean {
            node_key = Some(k);
            break;
        }
    }

    if let Some(key) = node_key {
        // 1. Get Node Data needed for cleanup
        if let Some(node) = graph.nodes.get(key).cloned() {
            // 2. Identify Ports
            let all_ports: Vec<_> = node
                .inputs
                .iter()
                .chain(node.outputs.iter())
                .cloned()
                .collect();

            // 3. Find Connections to remove
            let mut connections_to_remove = Vec::new();
            for (cid, conn) in &graph.connections {
                if all_ports.contains(&conn.from) || all_ports.contains(&conn.to) {
                    connections_to_remove.push(cid);
                }
            }

            // 4. Remove Connections
            for cid in connections_to_remove {
                graph.connections.remove(cid);
            }

            // 5. Remove Ports
            for pid in all_ports {
                graph.ports.remove(pid);
            }

            // 6. Remove Node
            graph.remove_node(key);

            Ok("Deleted".to_string())
        } else {
            Err("Node found but failed to retrieve data".to_string())
        }
    } else {
        Err("Node ID match not found".to_string())
    }
}

#[tauri::command]
fn delete_connection(state: tauri::State<AppState>, id: String) -> Result<String, String> {
    let mut graph = state.graph.lock().unwrap();
    let id_clean = id.trim_matches('"');

    // Find Connection Key
    let mut conn_key = None;
    for (k, _) in &graph.connections {
        let serialized = serde_json::to_string(&k).map_err(|e| e.to_string())?;
        if serialized.trim_matches('"') == id_clean {
            conn_key = Some(k);
            break;
        }
    }

    if let Some(key) = conn_key {
        graph.connections.remove(key);
        Ok("Deleted".to_string())
    } else {
        Err("Connection ID match not found".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .manage(AppState {
            graph: Mutex::new(Graph::default()),
            registry: Mutex::new({
                let mut registry = DefinitionRegistry::default();
                let platforms_path =
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../platforms");
                println!("Loading platforms from: {:?}", platforms_path);
                if let Err(e) = registry.load_from_dir(&platforms_path) {
                    eprintln!("Failed to load platforms: {}", e);
                }
                registry
            }),
        })
        .invoke_handler(tauri::generate_handler![
            get_graph,
            get_node_templates,
            add_node,
            connect_ports,
            update_node_position,
            delete_node,
            delete_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
