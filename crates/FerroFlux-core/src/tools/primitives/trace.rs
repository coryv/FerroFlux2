use crate::api::events::SystemEvent;
use crate::tools::{Tool, ToolContext};
use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

pub struct TraceTool;

impl Tool for TraceTool {
    fn id(&self) -> &'static str {
        "trace"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let label = params
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Trace");
        let data = params.get("data").cloned().unwrap_or(Value::Null);

        // Use Trace ID from context
        let trace_id = context.trace_id.clone();

        // Emit Log event
        if let Some(bus) = context.event_bus.as_ref() {
            let _ = bus.0.send(SystemEvent::Log {
                level: "TRACE".to_string(),
                message: format!("{}: {}", label, data),
                trace_id,
                timestamp: Utc::now().timestamp_millis(),
            });
        }

        Ok(data)
    }
}
