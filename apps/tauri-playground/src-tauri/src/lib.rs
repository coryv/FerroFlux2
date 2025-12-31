mod commands;
mod engine;
mod state;
mod types;

use crate::state::AppState;
use flow_canvas::history::HistoryManager;
use flow_canvas::model::{GraphState, WireStyle};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (engine_tx, engine_rx) = mpsc::channel(32);

    // Start dedicated engine thread
    engine::spawn_engine_thread(engine_rx);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            engine_tx,
            graph: Arc::new(Mutex::new(GraphState::default())),
            history: Arc::new(Mutex::new(HistoryManager::default())),
            default_wire_style: Arc::new(Mutex::new(WireStyle::Cubic)),
            registry_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_sdk,
            commands::log_js,
            commands::get_graph,
            commands::add_node,
            commands::add_edge,
            commands::bring_to_front,
            commands::set_connection_wire_style,
            commands::set_all_connection_wire_styles,
            commands::update_node_position,
            commands::delete_items,
            commands::undo,
            commands::redo,
            commands::copy_items,
            commands::paste_items,
            commands::get_node_templates,
            commands::deploy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
