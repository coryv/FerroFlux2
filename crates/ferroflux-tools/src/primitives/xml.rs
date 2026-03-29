use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use quick_xml::de::from_str;

pub struct XmlTool;

impl Tool for XmlTool {
    fn id(&self) -> &'static str {
        "core.utils.xml"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let xml_str = params.get("xml")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'xml' parameter"))?;

        let json_val: Value = from_str(xml_str)?;
        
        Ok(json!({ "result": json_val }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_xml_to_json() {
        let tool = XmlTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = ToolContext {
            local: &mut local,
            memory: &mut memory,
            trace_id: "test".to_string(),
            event_bus: None,
            shadow_mode: false,
            shadow_masks: &masks,
            store: None,
            secrets: None,
        };
        let params = json!({
            "xml": "<root><child>hello</child></root>"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"]["child"]["$text"], "hello");
    }
}
