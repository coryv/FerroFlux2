use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct EmitTool;

impl Tool for EmitTool {
    fn id(&self) -> &'static str {
        "emit"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let port = params
            .get("port")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'port'"))?;
        let value = params.get("value").unwrap_or(&Value::Null); // Default to Null (signal)

        // Write to a special reserved space in local context for outputs
        // The engine will read `_outputs` after execution.
        let outputs = context
            .local
            .entry("_outputs".to_string())
            .or_insert_with(|| serde_json::json!({}));

        if let Some(out_obj) = outputs.as_object_mut() {
            out_obj.insert(port.to_string(), value.clone());
        }

        Ok(Value::Null)
    }
}
