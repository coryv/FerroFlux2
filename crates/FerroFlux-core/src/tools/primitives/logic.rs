use crate::tools::primitives::utils::evaluate_condition;
use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct LogicTool;

impl Tool for LogicTool {
    fn id(&self) -> &'static str {
        "logic"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let data = params.get("data").unwrap_or(&Value::Null);
        let rules = params
            .get("rules")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Missing 'rules' array"))?;

        // Iterate rules in order. First match wins.
        for rule in rules {
            let output = rule
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            let condition = rule
                .get("condition")
                .ok_or_else(|| anyhow!("Missing 'condition' in rule"))?;

            if evaluate_condition(condition, data) {
                return Ok(serde_json::json!({ "match": output }));
            }
        }

        // No match
        Ok(serde_json::json!({ "match": "default" }))
    }
}
