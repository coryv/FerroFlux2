use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents observable events within the system runtime.
///
/// These events are broadcast via the `SystemEventBus` and can be consumed by
/// API clients (via SSE) or internal monitoring tools.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SystemEvent {
    /// A structured log message from a system component.
    Log {
        level: String,
        message: String,
        trace_id: String,
        timestamp: i64,
    },
    /// Represents a change in state or thought process of an AI Agent.
    AgentActivity {
        node_id: Uuid,
        activity: String,
        content: String,
    },
    /// Telemetry data for node execution performance and status.
    NodeTelemetry {
        trace_id: String,
        node_id: Uuid,
        node_type: String,
        execution_ms: u64,
        success: bool,
        details: serde_json::Value,
    },
    /// Updates on the lifecycle state of a workflow execution.
    WorkflowUpdate {
        id: Uuid,
        status: String,
    },
    /// Emitted when a workflow execution is paused at a checkpoint.
    CheckpointCreated {
        token: String,
        node_id: Uuid,
        trace_id: String,
    },
    /// Critical errors occurring during node execution.
    NodeError {
        trace_id: String,
        node_id: Uuid,
        error: String,
        timestamp: i64,
    },
    /// Represents the movement of data between two nodes in the graph.
    EdgeTraversal {
        source_id: Uuid,
        target_id: Uuid,
        timestamp: i64,
    },
    /// An incremental token/chunk from a streaming HTTP response (e.g. LLM token stream).
    StreamChunk {
        trace_id: String,
        step_id: String,
        chunk: String,
        done: bool,
    },
}

/// A Bevy Resource wrapper around a broadcast sender for system events.
///
/// This serves as the central nervous system for real-time feedback, allowing systems
/// to emit events that are propagated to the API layer (SSE) and other listeners.
#[derive(bevy_ecs::prelude::Resource, Clone)]
pub struct SystemEventBus(pub tokio::sync::broadcast::Sender<SystemEvent>);
