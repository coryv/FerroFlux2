use crate::tools::{Tool, ToolContext};
use anyhow::Result;
use serde_json::Value;

pub struct SleepTool;

impl Tool for SleepTool {
    fn id(&self) -> &'static str {
        "sleep"
    }
    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let ms = params
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
        Ok(Value::Null)
    }
}
