use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::Result;
use serde_json::Value;

/// Tool that accumulates items into a batch across multiple node executions.
///
/// Uses `global_memory` keyed by `(node_id, trace_id)` to maintain state.
pub struct AggregateTool;

impl Tool for AggregateTool {
    fn id(&self) -> &'static str {
        "aggregate"
    }
    
    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let item = params.get("item").cloned().unwrap_or(Value::Null);
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);
        
        // Memory key isolates by both trace (workflow instance) and node (specific step)
        let memory_key = format!("{}_{}_agg", context.trace_id, context.node_id);
        
        let mut batch = if let Some(Value::Array(arr)) = context.memory.get(&memory_key) {
            arr.clone()
        } else {
            Vec::new()
        };
        
        // Accumulate non-null items
        if !item.is_null() {
            batch.push(item);
        }
        
        let is_ready = batch.len() >= limit as usize;
        
        let result_batch = if is_ready {
            // Take the batch and clear memory
            let res = batch.clone();
            context.memory.remove(&memory_key);
            res
        } else {
            // Store updated batch back in memory
            context.memory.insert(memory_key, Value::Array(batch));
            Vec::new()
        };
        
        Ok(serde_json::json!({
            "batch": Value::Array(result_batch),
            "is_ready": is_ready
        }))
    }
}
