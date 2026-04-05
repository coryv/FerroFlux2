use crate::types::{ChatMessage, ChatRole, ToolDefinition};

/// Formatter for the Gemma 4 Prompt Template.
/// Implements the turn-based structure: <|turn>role\n...<turn|>\n
pub struct PromptTemplate;

impl PromptTemplate {
    /// Formats the final prompt string for the model from a list of messages.
    /// Injects <|think|> in the first system turn when enabled.
    /// Strips older thinking logs from the message history.
    pub fn format(
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
        enable_thinking: bool,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str("<bos>"); // Beginning of sequence

        // Handle System Turn
        let (first_system_msg, other_messages) = if !messages.is_empty() && messages[0].role == ChatRole::System {
            (Some(&messages[0]), &messages[1..])
        } else {
            (None, messages)
        };

        // Injected Thinking + Tool Definitions in a system turn
        if enable_thinking || tools.is_some() || first_system_msg.is_some() {
            prompt.push_str("<|turn>system\n");

            if enable_thinking {
                prompt.push_str("<|think|>");
            }

            if let Some(msg) = first_system_msg {
                prompt.push_str(msg.content.trim());
            }

            if let Some(tools) = tools {
                for tool in tools {
                    prompt.push_str("<|tool>");
                    prompt.push_str(&Self::format_tool(tool));
                    prompt.push_str("<tool|>");
                }
            }

            prompt.push_str("<turn|>\n");
        }

        // Handle remaining Messages
        for msg in other_messages {
            let role = match msg.role {
                ChatRole::System => "system",
                ChatRole::User => "user",
                ChatRole::Model => "model",
            };

            prompt.push_str(&format!("<|turn>{role}\n"));

            // Tool Calls output from model
            if let Some(tool_calls) = &msg.tool_calls {
                for call in tool_calls {
                    prompt.push_str(&format!("<|tool_call>call:{}", call.name));
                    prompt.push_str(&Self::format_json_args(&call.arguments));
                    prompt.push_str("<tool_call|>");
                }
            }

            // Tool Responses input to model
            if let Some(tool_responses) = &msg.tool_responses {
                for resp in tool_responses {
                    prompt.push_str(&format!("<|tool_response>response:{}", resp.name));
                    prompt.push_str(&Self::format_json_args(&resp.response));
                    prompt.push_str("<tool_response|>");
                }
            }

            // Strip thinking from model history (model card recommendation)
            if msg.role == ChatRole::Model {
                prompt.push_str(Self::strip_thinking(&msg.content).as_str());
            } else {
                prompt.push_str(msg.content.trim());
            }

            prompt.push_str("<turn|>\n");
        }

        // Add the generation prompt
        prompt.push_str("<|turn>model\n");

        prompt
    }

    /// Recursively formats a tool definition into the model-specific syntax.
    fn format_tool(tool: &ToolDefinition) -> String {
        let mut out = format!("declaration:{}", tool.name);
        out.push_str("{description:");
        out.push_str(&Self::escape_string(&tool.description));

        if let Some(params) = tool.parameters.get("properties") {
            out.push_str(",parameters:{properties:{");
            out.push_str(&Self::format_params(params));
            out.push('}');
            
            if let Some(req) = tool.parameters.get("required") {
                out.push_str(",required:");
                out.push_str(&req.to_string());
            }
            
            out.push_str(",type:<|\"|>OBJECT<|\"|>");
        }

        out.push('}');
        out
    }

    fn format_params(params: &serde_json::Value) -> String {
        let mut parts = Vec::new();
        if let Some(obj) = params.as_object() {
            for (key, val) in obj {
                let mut p = format!("{}:{{", key);
                if let Some(desc) = val.get("description") {
                    p.push_str(&format!("description:{}", Self::escape_string(desc.as_str().unwrap_or(""))));
                    p.push(',');
                }
                if let Some(ty) = val.get("type") {
                    p.push_str(&format!("type:{}", Self::escape_string(ty.as_str().unwrap_or(""))));
                }
                p.push('}');
                parts.push(p);
            }
        }
        parts.join(",")
    }

    fn format_json_args(val: &serde_json::Value) -> String {
        // Simple stringification for now, template uses custom format for objects
        val.to_string() 
    }

    fn escape_string(s: &str) -> String {
        format!("<|\"|>{}<|\"|>", s)
    }

    fn strip_thinking(s: &str) -> String {
        // Removes <|channel>thought ... <channel|> blocks
        let mut result = String::new();
        let parts: Vec<&str> = s.split("<|channel>").collect();
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                result.push_str(part);
            } else if let Some(end_idx) = part.find("<channel|>") {
                result.push_str(&part[end_idx + 10..]);
            }
        }
        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChatMessage, ToolDefinition};
    use serde_json::json;

    #[test]
    fn test_basic_turn() {
        let messages = vec![
            ChatMessage::new_user("Hello!".to_string()),
        ];
        let prompt = PromptTemplate::format(&messages, None, false);
        assert!(prompt.contains("<|turn>user\nHello!<turn|>\n"));
        assert!(prompt.contains("<|turn>model\n"));
        assert!(prompt.contains("<bos>"));
    }

    #[test]
    fn test_thinking_injection() {
        let messages = vec![
            ChatMessage::new_user("Go!".to_string()),
        ];
        let prompt = PromptTemplate::format(&messages, None, true);
        assert!(prompt.contains("<|turn>system\n<|think|><turn|>\n"));
    }

    #[test]
    fn test_history_stripping() {
        let messages = vec![
            ChatMessage::new_user("Hello".to_string()),
            ChatMessage::new_model("<|channel>thought\nI think...<channel|>\nHi there!".to_string()),
            ChatMessage::new_user("Again".to_string()),
        ];
        let prompt = PromptTemplate::format(&messages, None, false);
        // The thinking should be stripped from the model history
        assert!(prompt.contains("<|turn>model\nHi there!<turn|>\n"));
        assert!(!prompt.contains("I think..."));
    }

    #[test]
    fn test_tool_declaration() {
        let tools = vec![
            ToolDefinition {
                name: "get_weather".to_string(),
                description: "Get weather for a city".to_string(),
                parameters: json!({
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "City name"
                        }
                    },
                    "required": ["city"]
                }),
            },
        ];
        let prompt = PromptTemplate::format(&[], Some(&tools), false);
        assert!(prompt.contains("<|tool>declaration:get_weather"));
        assert!(prompt.contains("description:<|\"|>Get weather for a city<|\"|>"));
        assert!(prompt.contains("city:{description:<|\"|>City name<|\"|>,type:<|\"|>string<|\"|>}"));
        assert!(prompt.contains("type:<|\"|>OBJECT<|\"|>}"));
    }
}
