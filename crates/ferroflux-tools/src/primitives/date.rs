use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use chrono::{DateTime, Utc};

pub struct DateTool;

impl Tool for DateTool {
    fn id(&self) -> &'static str {
        "core.utils.date"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("now");

        match operation {
            "now" => {
                let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("%Y-%m-%dT%H:%M:%SZ");
                Ok(json!({ "result": Utc::now().format(format).to_string() }))
            },
            "parse" => {
                let date_str = params.get("date")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing 'date' for parse"))?;
                let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("%Y-%m-%dT%H:%M:%SZ");
                let dt = chrono::NaiveDateTime::parse_from_str(date_str, format)?
                    .and_utc();
                Ok(json!({ "result": dt.to_rfc3339() }))
            },
            "diff" => {
                let date1_str = params.get("date1").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'date1'"))?;
                let date2_str = params.get("date2").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'date2'"))?;
                
                let dt1 = DateTime::parse_from_rfc3339(date1_str)?;
                let dt2 = DateTime::parse_from_rfc3339(date2_str)?;
                
                let duration = dt2.signed_duration_since(dt1);
                Ok(json!({
                    "result": {
                        "seconds": duration.num_seconds(),
                        "minutes": duration.num_minutes(),
                        "hours": duration.num_hours(),
                        "days": duration.num_days()
                    }
                }))
            },
            _ => Err(anyhow!("Unsupported operation: {}", operation)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_date_now() {
        let tool = DateTool;
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
            secrets: None, executor: None,
        };
        let params = json!({ "operation": "now" });
        let res = tool.run(&mut ctx, params).unwrap();
        assert!(res["result"].as_str().is_some());
    }

    #[test]
    fn test_date_parse() {
        let tool = DateTool;
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
            secrets: None, executor: None,
        };
        let params = json!({
            "operation": "parse",
            "date": "2023-10-27T10:00:00Z"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"], "2023-10-27T10:00:00+00:00");
    }

    #[test]
    fn test_date_diff() {
        let tool = DateTool;
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
            secrets: None, executor: None,
        };
        let params = json!({
            "operation": "diff",
            "date1": "2023-10-27T10:00:00Z",
            "date2": "2023-10-27T11:00:00Z"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"]["seconds"], 3600);
    }
}
