use ferroflux_core::nodes::register_core_nodes;
use ferroflux_core::nodes::yaml_factory::YamlNodeFactory;
use ferroflux_core::resources::registry::{DefinitionRegistry, NodeRegistry};
use flow_canvas::model::GraphState;
use std::sync::Mutex;
use tauri::Manager;

type Graph = GraphState<String>;

struct AppState {
    graph: Mutex<Graph>,
}

#[tauri::command]
fn get_graph(state: tauri::State<AppState>) -> Graph {
    // Return a clone of the graph state
    state.graph.lock().unwrap().clone()
}

#[tauri::command]
fn get_node_templates() -> Vec<ferroflux_core::traits::node_factory::NodeMetadata> {
    let mut def_registry = DefinitionRegistry::default();

    // Resolve platforms path relative to Cargo manifest
    let platforms_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../platforms");
    println!("DEBUG: Loading templates from {:?}", platforms_path);

    // Load definitions
    if let Err(e) = def_registry.load_from_dir(&platforms_path) {
        eprintln!("Error loading platforms: {}", e);
    }

    let mut node_registry = NodeRegistry::new();

    // Register core
    register_core_nodes(&mut node_registry);

    // Register YAML nodes
    for (id, def) in def_registry.definitions {
        node_registry.register(&id, Box::new(YamlNodeFactory::new(def)));
    }

    node_registry.list_templates()
}

// Add Node command (Mock implementation for now to satisfy interface)
#[tauri::command]
fn add_node(state: tauri::State<AppState>, template_id: String, x: f32, y: f32) -> String {
    let mut graph = state.graph.lock().unwrap();
    // In a real implementation, we would use a factory to create the node.
    // For now, let's just use flow_canvas directly to create a dummy node with the right ID.

    let node = flow_canvas::model::Node {
        id: Default::default(), // Will be set by insert_node
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: glam::Vec2::new(x, y),
        size: glam::Vec2::new(150.0, 100.0), // Default size
        inputs: vec![],
        outputs: vec![],
        data: template_id, // Store type ID as data for now
        flags: Default::default(),
        style: None,
    };

    let id = graph.insert_node(node);
    // Return the SLOTMAP ID as string, or UUID?
    // The frontend expects a string ID. GraphState uses SlotMap NodeId.
    // Serializable NodeId converts to string in JSON.
    // But we should return the UUID if that's what we use for persistence?
    // flow_canvas uses NodeId for runtime.
    // Let's return the serialized NodeId.

    // We need to serialize the NodeId to string.
    // But NodeId::default() was used. graph.insert_node returns the new NodeId.
    // We can format it.
    format!("{:?}", id) // This format includes "NodeId(...)". We want the raw key?
                        // NodeId implements Serialize.
                        // But we are returning String.
                        // Let's return the key data or just debug format for now.
                        // Ideally we should return serde_json::to_string(&id).
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
        })
        .invoke_handler(tauri::generate_handler![
            get_graph,
            get_node_templates,
            add_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
