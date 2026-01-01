#![cfg(test)]

use crate::tools::primitives::{EmitTool, JsonQueryTool, SwitchTool};
use crate::tools::{Tool, ToolContext};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_switch_tool() {
    let tool = SwitchTool;
    let mut local = HashMap::new();
    let mut memory = HashMap::new();
    let mut context = ToolContext {
        local: &mut local,
        memory: &mut memory,
    };

    // Case 1: Match
    let params = json!({
        "value": "FOO",
        "cases": [
            { "condition": "FOO", "output": "branch_a" },
            { "condition": "default", "output": "branch_b" }
        ]
    });
    let res = tool.run(&mut context, params).unwrap();
    assert_eq!(res["branch"], "branch_a");

    // Case 2: Default
    let params_default = json!({
        "value": "BAR",
        "cases": [
            { "condition": "FOO", "output": "branch_a" },
            { "condition": "default", "output": "branch_b" }
        ]
    });
    let res_default = tool.run(&mut context, params_default).unwrap();
    assert_eq!(res_default["branch"], "branch_b");
}

#[test]
fn test_json_query_tool() {
    let tool = JsonQueryTool;
    let mut local = HashMap::new();
    let mut memory = HashMap::new();
    let mut context = ToolContext {
        local: &mut local,
        memory: &mut memory,
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
    };

    let params = json!({
        "port": "Success",
        "value": { "status": "ok" }
    });

    let _ = tool.run(&mut context, params).unwrap();

    // Check context for _outputs
    let outputs = context.local.get("_outputs").unwrap();
    assert_eq!(outputs["Success"]["status"], "ok");
}
