use crate::persistence::SaveFile;
use flow_canvas::model::GraphState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Top-level message envelope for the remote protocol.
///
/// Designed to be compatible with WebSocket or TCP text frames (JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ProtocolMessage<T> {
    Request(ClientRequest<T>),
    Response(ServerResponse<T>),
    Event(ServerEvent),
}

/// Requests initiating from the Client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum ClientRequest<T> {
    /// Handshake/Auth (Optional for V1)
    Connect {
        client_id: String,
    },

    /// Sync the visual graph state.
    SyncGraph(GraphState<T>),

    /// Control Execution
    Pause,
    Resume,
    Step(usize),

    /// Persistence
    SaveSnapshot, // Response will contain the snapshot
    LoadSnapshot(Box<SaveFile<T>>), // Request contains the snapshot to load

                                    // Inspection
                                    // Subscribe { topic: String }, // Future: targeted subscription
}

/// Responses from the Server (Engine).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ServerResponse<T> {
    Ok,
    Error(String),
    Snapshot(Box<SaveFile<T>>),
}

/// Unsolicited events pushed from the Server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "body")]
pub enum ServerEvent {
    Log {
        level: String,
        message: String,
    },
    Telemetry {
        node_id: Uuid,
        success: bool,
        execution_ms: u64,
        trace_id: String,
    },
    Pong,
}
