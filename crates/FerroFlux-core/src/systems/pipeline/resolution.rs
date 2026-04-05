use crate::components::execution_state::DataRef;
use crate::store::BlobStore;
use crate::systems::io::templating::{cel_json_func, cel_to_json, json_to_cel};
use anyhow::Result;
use cel_interpreter::{Context, Program};
use serde_json::Value;
use std::collections::HashMap;
/// Lazy pipeline execution context.
///
/// Instead of materializing all DataRefs upfront, `LazyCtx` holds the raw
/// DataRef map and a within-node cache. Blobs are only claimed from the store
/// when a CEL expression actually references them.
pub struct LazyCtx<'a> {
    /// The full context map for this node execution -- values may still be Blobs.
    pub data: &'a HashMap<String, DataRef>,
    /// BlobStore for claiming Blob variants on demand.
    pub store: Option<&'a BlobStore>,
    /// Within-node materialization cache. Once a key is claimed it lives here
    /// for the duration of this node's execution, avoiding repeat store calls.
    pub cache: &'a mut HashMap<String, Value>,
}

impl<'a> LazyCtx<'a> {
    /// Return the materialized `Value` for `key`, claiming from the BlobStore
    /// if needed. Returns `None` if the key is not in the context or the Blob
    /// cannot be claimed/deserialized.
    pub fn materialize_key(&mut self, key: &str) -> Option<Value> {
        if let Some(cached) = self.cache.get(key) {
            return Some(cached.clone());
        }
        let val = match self.data.get(key)? {
            DataRef::Inline(v) => v.clone(),
            DataRef::Blob(ticket) => {
                let store = self.store?;
                let bytes = store.claim(ticket).ok()?;
                serde_json::from_slice(&bytes).ok()?
            }
        };
        self.cache.insert(key.to_string(), val.clone());
        Some(val)
    }

    /// Invalidate a cache entry. Call this after mutating a key in `data` so
    /// subsequent expressions see the updated value.
    pub fn invalidate(&mut self, key: &str) {
        self.cache.remove(key);
    }
}

/// Recursively resolve CEL expressions within a JSON value against a lazy context.
///
/// - String values are compiled as CEL. If compilation succeeds, only the
///   top-level variables the expression actually references are materialized.
/// - Plain literals (e.g. "GET", "application/json") fail CEL compilation and
///   are returned as-is with no store interaction.
/// - Arrays and objects are resolved recursively.
pub fn resolve_recursive(value: &Value, ctx: &mut LazyCtx<'_>) -> Result<Value> {
    match value {
        Value::String(s) => {
            let s_ref = s.as_str();

            if let Some(stripped) = s_ref.strip_prefix("\\=") {
                return Ok(Value::String(format!("={stripped}")));
            }

            let expression = if let Some(stripped) = s_ref.strip_prefix('=') {
                stripped.trim()
            } else {
                // Not an explicit expression, treat as strict literal string.
                return Ok(Value::String(s.clone()));
            };

            if expression.is_empty() {
                return Ok(Value::String(String::new()));
            }

            let program = match Program::compile(expression) {
                Ok(p) => p,
                Err(e) => {
                    return Err(anyhow::anyhow!("CEL compile error for explicit expression '{}': {}", expression, e));
                }
            };

            // Determine exactly which top-level variables this expression uses,
            // then materialize only those -- leaving all other DataRefs untouched.
            let mut refs: Vec<String> = program
                .references()
                .variables()
                .into_iter()
                .map(|s| s.to_string())
                .collect();

            // Fallback: If no refs were found but the expression is a simple identifier,
            // CEL might not have picked it up. Try materializing it anyway.
            if refs.is_empty() && expression.chars().all(|c| c.is_alphanumeric() || c == '_') {
                refs.push(expression.to_string());
            }

            let mut partial = serde_json::Map::new();
            for key in &refs {
                if let Some(val) = ctx.materialize_key(key) {
                    partial.insert(key.clone(), val);
                }
            }

            let partial_val = Value::Object(partial);
            let mut cel_ctx = Context::default();
            cel_ctx.add_function("json", cel_json_func);
            if let Some(obj) = partial_val.as_object() {
                for (k, v) in obj {
                    let _ = cel_ctx.add_variable(k, json_to_cel(v.clone()));
                }
            }

            match program.execute(&cel_ctx) {
                Ok(cel_val) => Ok(cel_to_json(cel_val)),
                Err(e) => {
                    Err(anyhow::anyhow!("CEL execution error for explicit expression '{}': {}", expression, e))
                }
            }
        }

        Value::Array(arr) => {
            let mut new_arr = Vec::new();
            for v in arr {
                new_arr.push(resolve_recursive(v, ctx)?);
            }
            Ok(Value::Array(new_arr))
        }

        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), resolve_recursive(v, ctx)?);
            }
            Ok(Value::Object(new_obj))
        }

        _ => Ok(value.clone()),
    }
}

pub fn resolve_dataref_to_value(data: &DataRef, store: Option<&BlobStore>) -> Option<Value> {
    match data {
        DataRef::Inline(val) => Some(val.clone()),
        DataRef::Blob(ticket) => {
            if let Some(store) = store {
                if let Ok(bytes) = store.claim(ticket) {
                    serde_json::from_slice(&bytes).ok()
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
