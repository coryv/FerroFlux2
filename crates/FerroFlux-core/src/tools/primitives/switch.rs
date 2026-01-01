use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct SwitchTool;

impl Tool for SwitchTool {
    fn id(&self) -> &'static str {
        "switch"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let value = params
            .get("value")
            .ok_or_else(|| anyhow!("Missing 'value'"))?;
        let cases = params
            .get("cases")
            .ok_or_else(|| anyhow!("Missing 'cases'"))?;

        // Simple equality check for now. Future: Rhai/Expr evaluation.
        if let Some(cases_arr) = cases.as_array() {
            for case in cases_arr {
                let condition = case.get("condition").and_then(|v| v.as_str()).unwrap_or("");
                let output = case.get("output").and_then(|v| v.as_str()).unwrap_or("");

                // Very basic string equality for MVP
                // "default" is a special keyword
                if condition == "default" {
                    return Ok(serde_json::json!({ "branch": output }));
                }

                // Check equality
                // Avoid to_string allocation for strings
                #[allow(clippy::cmp_owned)]
                let is_match = match value {
                    Value::String(s) => s == condition,
                    _ => value.to_string() == condition,
                };

                if is_match {
                    return Ok(serde_json::json!({ "branch": output }));
                }
            }
        }

        Err(anyhow!("No matching case found"))
    }
}
