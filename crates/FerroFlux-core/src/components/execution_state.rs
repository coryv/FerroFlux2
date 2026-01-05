use crate::store::blob::SecureTicket;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A reference to data that can be either inline or stored in the BlobStore.
/// This implements the "Manifest Pattern" to keep the workflow state lightweight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataRef {
    Inline(Value),
    Blob(SecureTicket),
}

impl DataRef {
    /// Helper to get a reference to the inner Value if inline,
    /// or returns None if it's a blob (caller must resolve).
    pub fn as_inline(&self) -> Option<&Value> {
        match self {
            DataRef::Inline(v) => Some(v),
            DataRef::Blob(_) => None,
        }
    }
}

/// Component that holds the runtime state of an active workflow execution.
/// This IS the "Enriched Bundle" or "Flow Bus".
#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ActiveWorkflowState {
    /// The cumulative data context.
    /// Accessible in templates via `{{ variable_name }}`.
    pub context: HashMap<String, DataRef>,

    /// Execution history/trace for debugging (Optional for now)
    pub history: Vec<String>,
}

impl ActiveWorkflowState {
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Merges update_data into the context.
    /// If update_data is an Object, its keys are merged at the root level.
    /// Merges update_data into the context.
    /// If update_data is an Object, its keys are merged at the root level.
    /// Values are stored as DataRef::Inline.
    pub fn merge(&mut self, update_data: Value) {
        if let Value::Object(map) = update_data {
            for (k, v) in map {
                self.context.insert(k, DataRef::Inline(v));
            }
        } else {
            // If it's not an object (e.g. primitive array/string), we can't merge at root.
            // In the future we might want a "default" key for this case,
            // but for now we expect output_transforms to return Objects.
            eprintln!(
                "WARN: Attempted to merge non-object into WorkflowState: {:?}",
                update_data
            );
        }
    }

    /// Sets a specific key in the context (aliasing).
    pub fn set(&mut self, key: &str, value: Value) {
        self.context.insert(key.to_string(), DataRef::Inline(value));
    }

    /// Sets a data reference directly (e.g. for avoiding serialization roundtrips)
    pub fn set_ref(&mut self, key: &str, data_ref: DataRef) {
        self.context.insert(key.to_string(), data_ref);
    }

    pub fn get(&self, key: &str) -> Option<&DataRef> {
        self.context.get(key)
    }
}
