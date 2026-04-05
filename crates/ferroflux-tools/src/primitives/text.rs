use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use regex::Regex;

pub struct TextTool;

impl Tool for TextTool {
    fn id(&self) -> &'static str {
        "core.utils.text"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let input = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("match");

        match operation {
            "match" => {
                let pattern = params.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'pattern'"))?;
                let re = Regex::new(pattern)?;
                let matches: Vec<String> = re.find_iter(input).map(|m| m.as_str().to_string()).collect();
                Ok(json!({ "result": matches }))
            },
            "replace" => {
                let pattern = params.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'pattern'"))?;
                let replacement = params.get("replacement").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'replacement'"))?;
                let re = Regex::new(pattern)?;
                let res = re.replace_all(input, replacement).to_string();
                Ok(json!({ "result": res }))
            },
            "slugify" => {
                let slug = input.to_lowercase()
                    .replace(|c: char| !c.is_alphanumeric(), "-")
                    .split('-')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("-");
                Ok(json!({ "result": slug }))
            },
            "truncate" => {
                let length = params.get("length").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                let truncated = if input.len() > length {
                    format!("{}...", &input[..length])
                } else {
                    input.to_string()
                };
                Ok(json!({ "result": truncated }))
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
    fn test_text_match() {
        let tool = TextTool;
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
            "operation": "match",
            "text": "hello world",
            "pattern": r"\w+"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"][0], "hello");
        assert_eq!(res["result"][1], "world");
    }

    #[test]
    fn test_text_slugify() {
        let tool = TextTool;
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
            "operation": "slugify",
            "text": "Hello World! This is a test."
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"], "hello-world-this-is-a-test");
    }

    #[test]
    fn test_text_truncate() {
        let tool = TextTool;
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
            "operation": "truncate",
            "text": "this is a long string",
            "length": 4
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"], "this...");
    }
}
