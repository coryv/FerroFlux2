use crate::tools::{Tool, ToolContext};
use anyhow::Result;
use serde_json::Value;

pub struct MathTool;

impl Tool for MathTool {
    fn id(&self) -> &'static str {
        "math"
    }
    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let a = params.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = params.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let op = params.get("op").and_then(|v| v.as_str()).unwrap_or("add");

        let res = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" => {
                if b != 0.0 {
                    a / b
                } else {
                    f64::MAX
                }
            }
            _ => 0.0,
        };
        Ok(serde_json::json!({ "result": res }))
    }
}
