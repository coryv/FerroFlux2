use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
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
        /// standard log level (info, warn, error)
        level: String,
        /// The log content
        message: String,
        /// Correlation ID for the execution flow
        trace_id: String,
        /// Unix timestamp in milliseconds
        timestamp: i64,
    },
    /// Represents a change in state or thought process of an AI Agent.
    AgentActivity {
        /// The UUID of the agent node
        node_id: Uuid,
        /// The type of activity (e.g., "Thinking", "Tool Call", "Final Answer")
        activity: String,
        /// The content or payload of the activity
        content: String,
    },
    /// Telemetry data for node execution performance and status.
    NodeTelemetry {
        /// correlation ID for the execution flow
        trace_id: String,
        /// The UUID of the executed node
        node_id: Uuid,
        /// The type/category of the node
        node_type: String, // "Agent", "Http", etc.
        /// Execution duration in milliseconds
        execution_ms: u64,
        /// Whether execution completed successfully
        success: bool,
        /// Additional context or return values
        details: serde_json::Value,
    },
    /// Updates on the lifecycle state of a workflow execution.
    WorkflowUpdate {
        /// The UUID of the workflow
        id: Uuid,
        /// The new status (e.g., "Running", "Completed", "Failed")
        status: String,
    },
    /// Emitted when a workflow execution is paused at a checkpoint.
    CheckpointCreated {
        /// The resumption token generated for this checkpoint
        token: String,
        /// The UUID of the node triggering the checkpoint
        node_id: Uuid,
        /// Correlation ID
        trace_id: String,
    },
    /// Critical errors occurring during node execution.
    NodeError {
        /// Correlation ID
        trace_id: String,
        /// The UUID of the failing node
        node_id: Uuid,
        /// Human-readable error message
        error: String,
        /// Unix timestamp in milliseconds
        timestamp: i64,
    },
    /// Represents the movement of data between two nodes in the graph.
    EdgeTraversal {
        /// The UUID of the upstream source node
        source_id: Uuid,
        /// The UUID of the downstream target node
        target_id: Uuid,
        /// Unix timestamp in milliseconds
        timestamp: i64,
    },
}

/// A Bevy Resource wrapper around a broadcast sender for system events.
///
/// This serves as the central nervous system for real-time feedback, allowing systems
/// to emit events that are propagated to the API layer (SSE) and other listeners.
#[derive(Resource, Clone)]
pub struct SystemEventBus(pub broadcast::Sender<SystemEvent>);
