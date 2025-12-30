//! # FerroFlux Inspector
//!
//! This crate provides the headless logic for the node configuration inspector.
//! It bridges the gap between the engine's node definitions and the visual UI.

pub mod schema;

use schema::VisualField;
use serde_json::Value;
use std::collections::HashMap;

/// The state of the inspector for a specific node.
#[derive(Debug, Default)]
pub struct InspectorState {
    /// The fields to display in the UI.
    pub fields: Vec<VisualField>,
    /// The current values of each field.
    pub values: HashMap<String, Value>,
    /// Validation errors for each field.
    pub errors: HashMap<String, String>,
}

impl InspectorState {
    pub fn new(fields: Vec<VisualField>, initial_values: HashMap<String, Value>) -> Self {
        Self {
            fields,
            values: initial_values,
            errors: HashMap::new(),
        }
    }

    /// Updates the value of a field and performs validation.
    pub fn update_value(&mut self, field_id: &str, value: Value) {
        self.values.insert(field_id.to_string(), value);
        self.validate_field(field_id);
    }

    /// Validates a specific field.
    pub fn validate_field(&mut self, field_id: &str) {
        if let Some(field) = self.fields.iter().find(|f| f.id == field_id) {
            if field.required && self.values.get(field_id).map_or(true, |v| v.is_null()) {
                self.errors
                    .insert(field_id.to_string(), "Required".to_string());
            } else {
                self.errors.remove(field_id);
            }
        }
    }

    /// Returns true if all fields are valid.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}
