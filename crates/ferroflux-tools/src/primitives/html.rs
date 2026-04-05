use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use scraper::{Html, Selector};

pub struct HtmlTool;

impl Tool for HtmlTool {
    fn id(&self) -> &'static str {
        "core.utils.html"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let html_str = params.get("html").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'html'"))?;
        let selector_str = params.get("selector").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'selector'"))?;
        let attr = params.get("attr").and_then(|v| v.as_str());

        let document = Html::parse_document(html_str);
        let selector = Selector::parse(selector_str).map_err(|e| anyhow!("Invalid selector: {}", e))?;

        let results: Vec<String> = document.select(&selector)
            .map(|element| {
                if let Some(a) = attr {
                    element.value().attr(a).unwrap_or("").to_string()
                } else {
                    element.text().collect::<Vec<_>>().join("")
                }
            })
            .collect();

        Ok(json!({ "result": results }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_html_selector_text() {
        let tool = HtmlTool;
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
            "html": "<html><body><h1>Hello World</h1><p>Test paragraph</p></body></html>",
            "selector": "h1"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"][0], "Hello World");
    }

    #[test]
    fn test_html_selector_attr() {
        let tool = HtmlTool;
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
            "html": "<html><body><a href='https://example.com'>Link</a></body></html>",
            "selector": "a",
            "attr": "href"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"][0], "https://example.com");
    }
}
