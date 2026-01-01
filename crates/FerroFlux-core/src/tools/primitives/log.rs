use crate::tools::{Tool, ToolContext};
use anyhow::Result;
use serde_json::Value;

pub struct LogTool;

impl Tool for LogTool {
    fn id(&self) -> &'static str {
        "log"
    }
    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let level = params
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO");
        let msg = params.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let data = params.get("data");

        println!("[{}] {}: {:?}", level, msg, data);
        Ok(Value::Null)
    }
}
