use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for a Human-in-the-Loop Checkpoint.
///
/// Pauses execution until an external signal (via API or Webhook) resumes it.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Max time to wait before auto-expiring or failing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}
