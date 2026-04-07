use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use base64::{Engine as _, engine::general_purpose};

pub struct PdfReadTool;

impl Tool for PdfReadTool {
    fn id(&self) -> &'static str {
        "core.utils.pdf_read"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let pdf_data = if let Some(base64_str) = params.get("pdf_base64").and_then(|v| v.as_str()) {
            general_purpose::STANDARD.decode(base64_str)?
        } else if let Some(bytes) = params.get("pdf_bytes").and_then(|v| v.as_array()) {
            bytes.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect()
        } else {
            return Err(anyhow!("Missing 'pdf_base64' or 'pdf_bytes'"));
        };

        let text = pdf_extract::extract_text_from_mem(&pdf_data)
            .map_err(|e| anyhow!("Failed to extract text from PDF: {}", e))?;

        Ok(json!({ "result": text }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_pdf_read_error_on_invalid_data() {
        let tool = PdfReadTool;
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
            "pdf_base64": "not-a-pdf"
        });
        let res = tool.run(&mut ctx, params);
        assert!(res.is_err());
    }
}
