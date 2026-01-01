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

pub fn evaluate_condition(condition: &Value, data: &Value) -> bool {
    // Check for logical groups "AND" / "OR" (or lowercase)
    if let Some(rules) = condition.get("rules").and_then(|v| v.as_array()) {
        let op = condition
            .get("operator")
            .and_then(|v| v.as_str())
            .unwrap_or("AND")
            .to_uppercase();

        if op == "OR" {
            // Any rule must be true
            for r in rules {
                if evaluate_condition(r, data) {
                    return true;
                }
            }
            return false;
        } else {
            // AND (default): All rules must be true
            for r in rules {
                if !evaluate_condition(r, data) {
                    return false;
                }
            }
            return true;
        }
    }

    // Leaf condition
    let field = condition
        .get("field")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // Use json primitives for direct access if field is simple, or pointer if starts with /
    let val_in_data = if field.starts_with('/') {
        data.pointer(field).unwrap_or(&Value::Null)
    } else if !field.is_empty() {
        data.get(field).unwrap_or(&Value::Null)
    } else {
        &Value::Null
    };

    let op = condition
        .get("operator")
        .and_then(|v| v.as_str())
        .unwrap_or("==");
    let target_val = condition.get("value").unwrap_or(&Value::Null);

    compare_values(val_in_data, op, target_val)
}

pub fn compare_values(a: &Value, op: &str, b: &Value) -> bool {
    match op {
        "==" => a == b,
        "!=" => a != b,
        ">" => {
            if let (Some(na), Some(nb)) = (a.as_f64(), b.as_f64()) {
                na > nb
            } else {
                false
            }
        }
        "<" => {
            if let (Some(na), Some(nb)) = (a.as_f64(), b.as_f64()) {
                na < nb
            } else {
                false
            }
        }
        ">=" => {
            if let (Some(na), Some(nb)) = (a.as_f64(), b.as_f64()) {
                na >= nb
            } else {
                false
            }
        }
        "<=" => {
            if let (Some(na), Some(nb)) = (a.as_f64(), b.as_f64()) {
                na <= nb
            } else {
                false
            }
        }
        "contains" => match a {
            Value::String(s) => b.as_str().map(|bs| s.contains(bs)).unwrap_or(false),
            Value::Array(arr) => arr.contains(b),
            _ => false,
        },
        "starts_with" => {
            if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
                sa.starts_with(sb)
            } else {
                false
            }
        }
        "ends_with" => {
            if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
                sa.ends_with(sb)
            } else {
                false
            }
        }
        _ => false, // Unknown operator
    }
}
