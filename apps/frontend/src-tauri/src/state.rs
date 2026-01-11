use crate::engine::EngineCommand;
use crate::types::PlaygroundNodeData;
use flow_canvas::history::HistoryManager;
use flow_canvas::model::{GraphState, WireStyle};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AppState {
    pub engine_tx: mpsc::Sender<EngineCommand>,
    pub graph: Arc<Mutex<GraphState<PlaygroundNodeData>>>,
    pub history: Arc<Mutex<HistoryManager<PlaygroundNodeData>>>,
    pub default_wire_style: Arc<Mutex<WireStyle>>,
    pub registry_cache: Arc<Mutex<HashMap<String, crate::types::NodeTemplate>>>,
}
