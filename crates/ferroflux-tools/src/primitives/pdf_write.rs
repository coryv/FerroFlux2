use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use printpdf::*;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};

pub struct PdfWriteTool;

impl Tool for PdfWriteTool {
    fn id(&self) -> &'static str {
        "core.utils.pdf_write"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let title = params.get("title").and_then(|v| v.as_str()).unwrap_or("Document");
        let content = params.get("content").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'content'"))?;

        let (doc, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Add some basic text
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        current_layer.use_text(content, 12.0, Mm(20.0), Mm(280.0), &font);

        let mut buf = std::io::BufWriter::new(Cursor::new(Vec::new()));
        doc.save(&mut buf)?;

        let base64_pdf = general_purpose::STANDARD.encode(buf.into_inner()?.into_inner());
        Ok(json!({ "result": base64_pdf }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_pdf_write_basic() {
        let tool = PdfWriteTool;
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
            "title": "Test PDF",
            "content": "Hello FerroFlux"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert!(res["result"].as_str().is_some());
    }
}
