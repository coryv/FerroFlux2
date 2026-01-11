mod commands;
mod engine;
mod state;
mod types;

use crate::state::AppState;
use flow_canvas::history::HistoryManager;
use flow_canvas::model::{GraphState, WireStyle};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{mpsc, Mutex};
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (engine_tx, engine_rx) = mpsc::channel(32);

    // Start dedicated engine thread
    engine::spawn_engine_thread(engine_rx);

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            apply_vibrancy(
                &window,
                NSVisualEffectMaterial::UnderWindowBackground,
                None,
                None,
            )
            .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            Ok(())
        })
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
            commands::deploy,
            commands::update_node_settings,
            commands::reload_definitions,
            commands::simulate_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
