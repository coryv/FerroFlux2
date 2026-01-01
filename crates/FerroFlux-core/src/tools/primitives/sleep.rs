use crate::tools::{Tool, ToolContext};
use anyhow::Result;
use serde_json::Value;

pub struct SleepTool;

impl Tool for SleepTool {
    fn id(&self) -> &'static str {
        "sleep"
    }
    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        // Shadow Mode: Check for override or skip
        if context.shadow_mode {
            let mock = context.shadow_masks.get(self.id());
            if let Some(cfg) = mock {
                // If mock specifies delay, use that instead of param
                if cfg.delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(cfg.delay_ms));
                }
                return Ok(cfg.return_value.clone());
            }
            // If no mock, we might want to skip the sleep for speed?
            // Or respect it? Let's respect it for realism, but maybe log it.
            tracing::info!(
                "Shadow Mode: Sleeping for {}ms",
                params
                    .get("duration_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            );
        }

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
