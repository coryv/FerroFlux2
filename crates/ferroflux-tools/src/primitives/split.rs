use ferroflux_types::tool::{Tool, ToolContext};
use ferroflux_types::DataRef;
use anyhow::Result;
use serde_json::Value;

pub struct SplitTool;

impl Tool for SplitTool {
    fn id(&self) -> &'static str {
        "split"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let array = match params.get("array").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => {
                // If the array is missing or invalid, we assume we have nothing to do.
                // This happens in partial triggers before data has arrived.
                return Ok(serde_json::json!({
                    "current_item": Value::Null,
                    "is_done": true,
                    "index": 0
                }));
            }
        };

        // We use a internal key in the local context to track the iteration index.
        // This index is managed by the engine's Iterator loop or the tool itself
        // if called repeatedly within the same execution context.
        let index_key = "__iterator_index".to_string();
        
        let current_index = context
            .local
            .get(&index_key)
            .and_then(|dr| match dr {
                DataRef::Inline(v) => v.as_u64(),
                _ => None,
            })
            .unwrap_or(0) as usize;

        if current_index >= array.len() {
            return Ok(serde_json::json!({
                "current_item": Value::Null,
                "is_done": true,
                "index": current_index
            }));
        }

        let item = array[current_index].clone();
        let is_done = current_index == array.len() - 1;

        // Increment index for the next call in this same node execution (if any)
        context.local.insert(index_key, DataRef::Inline(serde_json::json!(current_index + 1)));

        Ok(serde_json::json!({
            "current_item": item,
            "is_done": is_done,
            "index": current_index
        }))
    }
}
