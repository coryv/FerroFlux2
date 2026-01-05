#![cfg(test)]

use crate::tools::primitives::{EmitTool, JsonQueryTool, LogicTool};
use crate::tools::{Tool, ToolContext};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_logic_tool_basic_switch() {
    let tool = LogicTool;
    let mut local = HashMap::new();
    let mut memory = HashMap::new();
    let mut context = ToolContext {
        local: &mut local,
        memory: &mut memory,
        trace_id: "test-trace".to_string(),
        event_bus: None,
        shadow_mode: false,
        shadow_masks: &HashMap::new(),
        secret_store: None,
        runtime: None,
    };

    // Simulate "Switch" behavior: strict equality check on a field
    let data = json!({ "status": "FOO" });
    let params = json!({
        "data": data,
        "rules": [
            { "condition": { "field": "status", "value": "FOO", "operator": "==" }, "output": "branch_a" },
            { "condition": { "field": "status", "value": "BAR", "operator": "==" }, "output": "branch_b" }
        ]
    });

    let res = tool.run(&mut context, params).unwrap();
    assert_eq!(res["match"], "branch_a");

    // Test default fallback
    let data_unknown = json!({ "status": "BAZ" });
    let params_unknown = json!({
        "data": data_unknown,
        "rules": [
            { "condition": { "field": "status", "value": "FOO", "operator": "==" }, "output": "branch_a" }
        ]
    });
    let res_unknown = tool.run(&mut context, params_unknown).unwrap();
    assert_eq!(res_unknown["match"], "default");
}

#[test]
fn test_logic_tool_complex_operators() {
    let tool = LogicTool;
    let mut local = HashMap::new();
    let mut memory = HashMap::new();
    let mut context = ToolContext {
        local: &mut local,
        memory: &mut memory,
        trace_id: "test-trace".to_string(),
        event_bus: None,
        shadow_mode: false,
        shadow_masks: &HashMap::new(),
        secret_store: None,
        runtime: None,
    };

    let data = json!({
        "user": { "age": 25, "role": "admin" },
        "flags": ["beta", "priority"]
    });

    // Test Numeric GT and List Contains
    let params = json!({
        "data": data,
        "rules": [
            {
                "output": "high_value_user",
                "condition": {
                    "operator": "AND",
                    "rules": [
                        { "field": "/user/age", "operator": ">", "value": 18 },
                        { "field": "/flags", "operator": "contains", "value": "priority" }
                    ]
                }
            }
        ]
    });

    let res = tool.run(&mut context, params).unwrap();
    assert_eq!(res["match"], "high_value_user");
}

#[test]
fn test_json_query_tool() {
    let tool = JsonQueryTool;
    let mut local = HashMap::new();
    let mut memory = HashMap::new();
    let mut context = ToolContext {
        local: &mut local,
        memory: &mut memory,
        trace_id: "test-trace".to_string(),
        event_bus: None,
        shadow_mode: false,
        shadow_masks: &HashMap::new(),
        secret_store: None,
        runtime: None,
    };

    let data = json!({
        "foo": {
            "bar": "baz"
        }
    });

    // Case 1: Simple Key
    let params = json!({ "json": data, "path": "foo" });
    let res = tool.run(&mut context, params).unwrap();
    assert_eq!(res["result"]["bar"], "baz");

    // Case 2: Pointer
    let params_ptr = json!({ "json": data, "path": "/foo/bar" });
    let res_ptr = tool.run(&mut context, params_ptr).unwrap();
    assert_eq!(res_ptr["result"], "baz");
}

#[test]
fn test_emit_tool() {
    let tool = EmitTool;
    let mut local = HashMap::new();
    let mut memory = HashMap::new();
    let mut context = ToolContext {
        local: &mut local,
        memory: &mut memory,
        trace_id: "test-trace".to_string(),
        event_bus: None,
        shadow_mode: false,
        shadow_masks: &HashMap::new(),
        secret_store: None,
        runtime: None,
    };

    let params = json!({
        "port": "Success",
        "value": { "status": "ok" }
    });

    let _ = tool.run(&mut context, params).unwrap();

    // Check context for _outputs
    let outputs = context.local.get("_outputs").unwrap().as_inline().unwrap();
    assert_eq!(outputs["Success"]["status"], "ok");
}
