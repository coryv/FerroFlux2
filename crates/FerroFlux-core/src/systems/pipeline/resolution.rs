use crate::components::execution_state::DataRef;
use crate::store::BlobStore;
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::Value;

pub fn resolve_recursive(
    value: &Value,
    ctx: &Value,
    reg: &Handlebars,
    store: Option<&BlobStore>,
) -> Result<Value> {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            
            // 1. Shortcut Check: If the string is EXACTLY one tag, we can extract the raw object/value.
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                let inner_raw = &trimmed[2..trimmed.len() - 2];
                if !inner_raw.contains("{{") && !inner_raw.contains("}}") {
                    let inner = inner_raw.trim();
                    if !inner.starts_with("get ") {
                        if let Some(val) = lookup_path(ctx, inner, store) {
                            return Ok(val);
                        }
                    }
                }
            }

            // 2. Full Render: If not a shortcut, or lookup failed, use Handlebars.
            let mut flat_ctx = ctx.clone();
            if let Some(obj) = flat_ctx.as_object_mut() {
                // If we have a 'platform' object, merge its keys into the root if they don't collide
                let p_obj_opt = obj.get("platform").and_then(|v| v.as_object()).cloned();
                if let Some(p_obj) = p_obj_opt {
                    for (pk, pv) in p_obj {
                        if !obj.contains_key(&pk) {
                            obj.insert(pk, pv);
                        }
                    }
                }
            }

            let rendered = reg.render_template(s, &flat_ctx)?;
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
