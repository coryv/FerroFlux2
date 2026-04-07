use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct SwitchTool;

impl Tool for SwitchTool {
    fn id(&self) -> &'static str {
        "switch"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let value = params.get("value").ok_or_else(|| anyhow!("Missing 'value' to switch on"))?;
        let cases = params
            .get("cases")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Missing 'cases' array"))?;

        tracing::info!(switch_value = ?value, "SwitchTool: evaluating");

        let mut default_output = None;

        for case in cases {
            let condition = case.get("condition").ok_or_else(|| anyhow!("Missing 'condition' in case"))?;
            let output = case.get("output").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'output' string in case"))?;

            if condition == "default" {
                default_output = Some(output);
                continue;
            }

            let is_match = values_match(value, condition);
            tracing::info!(condition = ?condition, is_match = %is_match, "SwitchTool: checking case");

            // Simple equality check (handle string/number/bool comparison)
            if is_match {
                return Ok(serde_json::json!({ "branch": output }));
            }
        }

        if let Some(output) = default_output {
            return Ok(serde_json::json!({ "branch": output }));
        }

        // No match and no default
        Ok(serde_json::json!({ "branch": "default" }))
    }
}

fn values_match(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }

    if let (Some(a_f), Some(b_f)) = (to_f64(a), to_f64(b)) {
        return (a_f - b_f).abs() < f64::EPSILON;
    }

    let a_str = to_string_canonical(a);
    let b_str = to_string_canonical(b);

    a_str == b_str
}

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn to_string_canonical(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_switch_match() {
        let tool = SwitchTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = ToolContext {
            local: &mut local,
            memory: &mut memory,
            trace_id: "test".to_string(),
            node_id: "test_node".to_string(),
            tenant_id: "test_tenant".to_string(),
            event_bus: None,
            shadow_mode: false,
            shadow_masks: &masks,
            store: None,
            secrets: None,
            executor: None,
        };

        let params = serde_json::json!({
            "value": 200,
            "cases": [
                { "condition": 200, "output": "success" },
                { "condition": "default", "output": "error" }
            ]
        });

        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["branch"], "success");
    }

    #[test]
    fn test_switch_string_match() {
        let tool = SwitchTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = ToolContext {
            local: &mut local,
            memory: &mut memory,
            trace_id: "test".to_string(),
            node_id: "test_node".to_string(),
            tenant_id: "test_tenant".to_string(),
            event_bus: None,
            shadow_mode: false,
            shadow_masks: &masks,
            store: None,
            secrets: None,
            executor: None,
        };

        let params = serde_json::json!({
            "value": "200",
            "cases": [
                { "condition": "200", "output": "success" },
                { "condition": "default", "output": "error" }
            ]
        });

        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["branch"], "success");
    }

    #[test]
    fn test_switch_default() {
        let tool = SwitchTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = ToolContext {
            local: &mut local,
            memory: &mut memory,
            trace_id: "test".to_string(),
            node_id: "test_node".to_string(),
            tenant_id: "test_tenant".to_string(),
            event_bus: None,
            shadow_mode: false,
            shadow_masks: &masks,
            store: None,
            secrets: None,
            executor: None,
        };

        let params = serde_json::json!({
            "value": 404,
            "cases": [
                { "condition": 200, "output": "success" },
                { "condition": "default", "output": "error" }
            ]
        });

        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["branch"], "error");
    }
}
