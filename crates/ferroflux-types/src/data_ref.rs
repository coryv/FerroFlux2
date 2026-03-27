use crate::blob::{BlobStore, SecureTicket};
use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A reference to data that can be either inline or stored in the BlobStore.
///
/// This implements the "Manifest Pattern" to keep the workflow state lightweight:
/// small values are stored inline, large ones are offloaded to the `BlobStore`
/// and referenced here by `SecureTicket`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataRef {
    Inline(Value),
    Blob(SecureTicket),
}

impl DataRef {
    /// Returns the inner `Value` if this is an inline ref; `None` for blobs.
    pub fn as_inline(&self) -> Option<&Value> {
        match self {
            DataRef::Inline(v) => Some(v),
            DataRef::Blob(_) => None,
        }
    }
}

/// Component that holds the runtime state of an active workflow execution.
///
/// This is the "Enriched Bundle" / "Flow Bus" — it accumulates context values
/// as each node executes, making outputs available to downstream nodes via
/// template interpolation (e.g. `{{ variable_name }}`).
#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ActiveWorkflowState {
    /// The cumulative data context.
    pub context: HashMap<String, DataRef>,
    /// Execution history/trace for debugging.
    pub history: Vec<String>,
}

impl ActiveWorkflowState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges an object-typed `Value` into the context at the root level.
    pub fn merge(&mut self, update_data: Value) {
        if let Value::Object(map) = update_data {
            for (k, v) in map {
                self.context.insert(k, DataRef::Inline(v));
            }
        } else {
            tracing::warn!(
                "Attempted to merge non-object into WorkflowState: {:?}",
                update_data
            );
        }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.context.insert(key.to_string(), DataRef::Inline(value));
    }

    pub fn set_ref(&mut self, key: &str, data_ref: DataRef) {
        self.context.insert(key.to_string(), data_ref);
    }

    pub fn get(&self, key: &str) -> Option<&DataRef> {
        self.context.get(key)
    }

    /// Offloads any inline values that exceed `threshold_bytes` to the `BlobStore`.
    pub fn optimize_memory(&mut self, store: &BlobStore, threshold_bytes: usize) {
        for (key, data_ref) in self.context.iter_mut() {
            if let DataRef::Inline(val) = data_ref {
                let estimated_size = match val {
                    Value::String(s) => s.len(),
                    Value::Array(arr) => arr.len() * 100,
                    Value::Object(map) => map.len() * 100,
                    _ => 8,
                };

                if estimated_size > threshold_bytes
                    && let Ok(json_bytes) = serde_json::to_vec(val)
                        && let Ok(ticket) = store.check_in(&json_bytes)
                    {
                        *data_ref = DataRef::Blob(ticket);
                        tracing::debug!("Offloaded variable '{}' to blob storage", key);
                    }
            }
        }
    }
}
