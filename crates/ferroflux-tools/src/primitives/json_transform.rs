use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use jmespath;

pub struct JsonTransformTool;

impl Tool for JsonTransformTool {
    fn id(&self) -> &'static str {
        "core.utils.json"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let input = params.get("input").cloned().unwrap_or(json!({}));
        let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("query");

        match operation {
            "query" => {
                let expression = params.get("expression")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'expression' for query"))?;
                
                let expr = jmespath::compile(expression)?;
                let res = expr.search(&input)?;
                Ok(json!({ "result": *res }))
            },
            "merge" => {
                let mut base = input.clone();
                let other = params.get("other").ok_or_else(|| anyhow!("Missing 'other' for merge"))?;
                deep_merge(&mut base, other);
                Ok(json!({ "result": base }))
            },
            "flatten" => {
                let mut flattened = json!({});
                flatten_json(&input, "", &mut flattened);
                Ok(json!({ "result": flattened }))
            },
            _ => Err(anyhow!("Unsupported operation: {}", operation)),
        }
    }
}

fn deep_merge(base: &mut Value, other: &Value) {
    match (base, other) {
        (Value::Object(base_map), Value::Object(other_map)) => {
            for (k, v) in other_map {
                deep_merge(base_map.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (base, other) => {
            *base = other.clone();
        }
    }
}

fn flatten_json(input: &Value, prefix: &str, output: &mut Value) {
    match input {
        Value::Object(map) => {
            for (k, v) in map {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(v, &new_prefix, output);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{}.{}", prefix, i)
                };
                flatten_json(v, &new_prefix, output);
            }
        }
        _ => {
            if let Value::Object(map) = output {
                map.insert(prefix.to_string(), input.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_json_query() {
        let tool = JsonTransformTool;
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
        };
        let params = json!({
            "operation": "query",
            "expression": "foo.bar",
            "input": { "foo": { "bar": "baz" } }
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"], "baz");
    }

    #[test]
    fn test_json_merge() {
        let tool = JsonTransformTool;
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
        };
        let params = json!({
            "operation": "merge",
            "input": { "a": 1, "b": { "c": 2 } },
            "other": { "b": { "d": 3 }, "e": 4 }
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"]["b"]["c"], 2);
        assert_eq!(res["result"]["b"]["d"], 3);
        assert_eq!(res["result"]["e"], 4);
    }

    #[test]
    fn test_json_flatten() {
        let tool = JsonTransformTool;
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
        };
        let params = json!({
            "operation": "flatten",
            "input": { "a": 1, "b": { "c": 2, "d": [3, 4] } }
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"]["a"], 1);
        assert_eq!(res["result"]["b.c"], 2);
        assert_eq!(res["result"]["b.d.0"], 3);
        assert_eq!(res["result"]["b.d.1"], 4);
    }
}
