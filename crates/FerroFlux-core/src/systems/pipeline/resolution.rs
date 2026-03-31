use crate::components::execution_state::DataRef;
use crate::store::BlobStore;
use crate::systems::io::templating::{cel_to_json, json_to_cel};
use anyhow::Result;
use cel_interpreter::{Context, Program};
use serde_json::Value;

pub fn resolve_recursive(
    value: &Value,
    ctx: &Value,
    _reg: &Option<()>, // Placeholder for now, previously Handlebars
    _store: Option<&BlobStore>,
) -> Result<Value> {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            
            // Try to evaluate as CEL
            let mut context = Context::default();
            
            // Inject variables from context
            if let Some(obj) = ctx.as_object() {
                for (k, v) in obj {
                    let _ = context.add_variable(k, json_to_cel(v.clone()));
                }

            }

            match Program::compile(trimmed) {
                Ok(program) => {
                    match program.execute(&context) {
                        Ok(cel_val) => {
                            let json_val = cel_to_json(cel_val);
                            Ok(json_val)
                        }
                        Err(_) => {
                            // If it fails to execute (e.g. undefined variable), 
                            // maybe it was intended as a literal string.
                            Ok(Value::String(s.clone()))
                        }
                    }
                }
                Err(_) => {
                    // Not valid CEL, treat as literal
                    Ok(Value::String(s.clone()))
                }
            }
        }

        Value::Array(arr) => {
            let mut new_arr = Vec::new();
            for v in arr {
                new_arr.push(resolve_recursive(v, ctx, &None, _store)?);
            }
            Ok(Value::Array(new_arr))
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), resolve_recursive(v, ctx, &None, _store)?);
            }
            Ok(Value::Object(new_obj))
        }
        _ => Ok(value.clone()),
    }
}

pub fn lookup_path(
    ctx: &Value,
    path: &str,
    _store: Option<&BlobStore>, 
) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let mut current = ctx;

    for part in &parts {
        match current {
            Value::Object(map) => {
                if let Some(next) = map.get(*part) {
                    current = next;
                } else {
                    return None;
                }
            }
            Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    if let Some(next) = arr.get(idx) {
                        current = next;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current.clone())
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
