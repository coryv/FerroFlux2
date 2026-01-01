use bevy_ecs::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an execution flow (Trace).
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Trace(pub Uuid);

/// The currently active node in the trace traversal.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct TraceNode(pub Uuid);

/// Start time of the trace.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct TraceStart(pub DateTime<Utc>);

/// Snapshot of the initial input data for the trace.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct TraceInput(pub serde_json::Value);

/// Metadata for redaction or sensitivity.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Sensitive(pub bool);
