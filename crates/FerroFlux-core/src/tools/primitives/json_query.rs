use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct JsonQueryTool;

impl Tool for JsonQueryTool {
    fn id(&self) -> &'static str {
        "json_query"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let json = params
            .get("json")
            .ok_or_else(|| anyhow!("Missing 'json'"))?;
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'path'"))?;

        // Simple pointer access for MVP (e.g., "/foo/bar")
        // Future: full JSONPath
        if path.starts_with('/') {
            match json.pointer(path) {
                Some(v) => Ok(serde_json::json!({ "result": v })),
                None => Ok(serde_json::json!({ "result": Value::Null })),
            }
        } else {
            // Fallback for simple keys
            match json.get(path) {
                Some(v) => Ok(serde_json::json!({ "result": v })),
                None => Ok(serde_json::json!({ "result": Value::Null })),
            }
        }
    }
}
