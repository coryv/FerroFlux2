use crate::components::execution_state::DataRef;
use crate::store::BlobStore;
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;

pub fn resolve_recursive(
    value: &Value,
    ctx: &Value,
    reg: &Handlebars,
    store: Option<&BlobStore>,
) -> Result<Value> {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                let inner = trimmed[2..trimmed.len() - 2].trim();
                if !inner.contains(' ')
                    && !inner.starts_with("get ")
                    && let Some(val) = lookup_path(ctx, inner, store)
                {
                    return Ok(val);
                }
            }
            // Render template using lightweight context
            let rendered = reg.render_template(s, ctx)?;
            Ok(Value::String(rendered))
        }
        Value::Array(arr) => {
            let mut new_arr = Vec::new();
            for v in arr {
                new_arr.push(resolve_recursive(v, ctx, reg, store)?);
            }
            Ok(Value::Array(new_arr))
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), resolve_recursive(v, ctx, reg, store)?);
            }
            Ok(Value::Object(new_obj))
        }
        _ => Ok(value.clone()),
    }
}

pub fn lookup_path(
    ctx: &Value,
    path: &str,
    _store: Option<&BlobStore>, // No longer need store here as ctx is materialized
) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let mut current = ctx;

    for part in &parts {
        match current {
            Value::Object(map) => {
                current = map.get(*part)?;
            }
            Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    current = arr.get(idx)?;
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
        DataRef::Inline(v) => Some(v.clone()),
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
