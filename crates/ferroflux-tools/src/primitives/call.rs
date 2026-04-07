use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};

/// A tool that invokes another platform action dynamically.
/// 
/// This leverages the engine's 'ActionExecutor' to run YAML-defined
/// actions without duplicating their logic in Rust.
pub struct CallTool;

impl Tool for CallTool {
    fn id(&self) -> &'static str {
        "call"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let action_id = params.get("action_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'action_id'"))?;
        
        let sub_params = params.get("params").cloned().unwrap_or(json!({}));

        let executor = context.executor.ok_or_else(|| {
            anyhow!("ActionExecutor not available in ToolContext. Cannot perform dynamic calls.")
        })?;

        // Delegate execution to the engine's action runner
        let tenant_id = ferroflux_types::tenant::TenantId::from(context.tenant_id.as_str());
        let result = executor.execute(&tenant_id, action_id, sub_params, context)?;

        Ok(result)
    }
}
