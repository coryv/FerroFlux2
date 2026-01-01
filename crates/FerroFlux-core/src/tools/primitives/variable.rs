use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

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
