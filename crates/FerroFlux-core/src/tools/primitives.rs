use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

// --- Switch Tool ---
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
                let is_match = value
                    .as_str()
                    .map(|s| s == condition)
                    .unwrap_or_else(|| value.to_string() == condition);

                if is_match {
                    return Ok(serde_json::json!({ "branch": output }));
                }
            }
        }

        Err(anyhow!("No matching case found"))
    }
}

// --- JSON Query Tool ---
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

// --- Emit Tool ---
// Pushes a value to a named output port.
// In a real implementation, this would likely push to a channel or
// modify a specific part of the context that the engine reads later.
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

// --- HTTP Client Tool ---
// Requires `reqwest` (blocking for MVP logic inside sync tool, valid for async if tool run is async)
// NOTE: Tool trait signature is currently sync `run`. We might need to change it to async
// or use block_in_place if we are in a Tokio runtime.
// For now, let's assume async execution context if possible or use blocking reqwest.
// Given `anyhow::Result`, straightforward implementation.
pub struct HttpClientTool;
impl Tool for HttpClientTool {
    fn id(&self) -> &'static str {
        "http_client"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'url'"))?;
        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        let body = params.get("body");
        let headers_val = params.get("headers");

        // Use blocking client for now to match Sync trait signature
        // (Assuming this runs in a threadpool or blocking task)
        let client = reqwest::blocking::Client::new();

        let mut req = match method {
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => client.get(url),
        };

        if let Some(h) = headers_val.and_then(|v| v.as_object()) {
            for (k, v) in h {
                if let Some(s) = v.as_str() {
                    req = req.header(k, s);
                }
            }
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send()?;
        let status = resp.status().as_u16();
        let headers: std::collections::HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Attempt to parse JSON, fallback to text
        let body_val: Value = resp.json().unwrap_or(Value::Null);

        Ok(serde_json::json!({
            "status": status,
            "headers": headers,
            "body": body_val
        }))
    }
}
// --- Logic Tool ---
// Evaluates complex boolean logic against a data object.
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

fn evaluate_condition(condition: &Value, data: &Value) -> bool {
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

fn compare_values(a: &Value, op: &str, b: &Value) -> bool {
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

// --- Log Tool ---
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

// --- Sleep Tool ---
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

// --- Variable Tools ---
pub struct SetVarTool;
impl Tool for SetVarTool {
    fn id(&self) -> &'static str {
        "set_var"
    }
    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'name'"))?;
        let value = params.get("value").unwrap_or(&Value::Null);

        context.memory.insert(name.to_string(), value.clone());
        Ok(Value::Null)
    }
}

pub struct GetVarTool;
impl Tool for GetVarTool {
    fn id(&self) -> &'static str {
        "get_var"
    }
    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'name'"))?;
        let val = context.memory.get(name).unwrap_or(&Value::Null);

        Ok(serde_json::json!({ "value": val }))
    }
}

// --- Math Tool ---
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
