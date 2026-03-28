use anyhow::{anyhow, Result};
use lettre::message::{header, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use serde_json::{json, Value};
use std::collections::HashMap;

use ferroflux_types::tool::{Tool, ToolContext};

pub struct SmtpTool;

impl Tool for SmtpTool {
    fn id(&self) -> &'static str {
        "smtp"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let host = params
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'host'"))?;
        let port = params.get("port").and_then(|v| v.as_u64()).unwrap_or(587) as u16;
        let username = params.get("username").and_then(|v| v.as_str());
        let password = params.get("password").and_then(|v| v.as_str());
        let encryption = params
            .get("encryption")
            .and_then(|v| v.as_str())
            .unwrap_or("starttls");

        let from = params
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'from' address"))?;
        let to = params
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'to' address"))?;
        let subject = params
            .get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'subject'"))?;
        let body_text = params.get("body_text").and_then(|v| v.as_str());
        let body_html = params.get("body_html").and_then(|v| v.as_str());

        // 1. Build the Alternative body (Text and/or HTML)
        let base_body = match (body_text, body_html) {
            (Some(text), Some(html)) => MultiPart::alternative_plain_html(text.to_string(), html.to_string()),
            (Some(text), None) => MultiPart::alternative().singlepart(SinglePart::plain(text.to_string())),
            (None, Some(html)) => MultiPart::alternative().singlepart(SinglePart::html(html.to_string())),
            (None, None) => return Err(anyhow!("Email must have at least one body part (text or html)")),
        };

        // 2. Wrap in Mixed if there are attachments
        let attachments = params.get("attachments").and_then(|v| v.as_array());
        let final_body = if let Some(atts) = attachments && !atts.is_empty() {
            let mut mixed = MultiPart::mixed().multipart(base_body);
            for att in atts {
                let id_str = att.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Attachment missing 'id' ticket"))?;
                let uuid = uuid::Uuid::parse_str(id_str)?;
                
                let store = context.store.ok_or_else(|| anyhow!("BlobStore not available"))?;
                let data = store.claim(&ferroflux_types::blob::SecureTicket {
                    id: uuid,
                    metadata: HashMap::new(),
                })?;

                let filename = att.get("filename").and_then(|v| v.as_str()).unwrap_or("attachment");
                let content_type = att.get("content_type").and_then(|v| v.as_str()).unwrap_or("application/octet-stream");

                mixed = mixed.singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::parse(content_type)?)
                        .header(header::ContentDisposition::attachment(filename))
                        .body(data)
                );
            }
            mixed
        } else {
            base_body
        };

        // 3. Build and Send the Message
        let email = Message::builder()
            .from(from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(final_body)
            .map_err(|e| anyhow!("Failed to build email: {}", e))?;

        let mut transport_builder = if encryption == "none" {
            SmtpTransport::builder_dangerous(host.to_string())
        } else {
            SmtpTransport::relay(host)?
        };

        transport_builder = transport_builder.port(port);
        if let (Some(u), Some(p)) = (username, password) {
            transport_builder = transport_builder.credentials(Credentials::new(u.to_string(), p.to_string()));
        }

        let mailer = transport_builder.build();
        let result = mailer.send(&email).map_err(|e| anyhow!("SMTP error: {}", e))?;

        Ok(json!({
            "status": "sent",
            "response": format!("{:?}", result)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::Tool;

    #[test]
    fn test_smtp_tool_id() {
        let tool = SmtpTool;
        assert_eq!(tool.id(), "smtp");
    }
}

