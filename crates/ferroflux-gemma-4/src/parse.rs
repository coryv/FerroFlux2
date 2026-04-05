use crate::model_selector::ModelVariant;
use crate::types::{AgentResponse, ToolCall};

pub struct ResponseParser;

impl ResponseParser {
    /// Parses a raw string from the model output into the structured AgentResponse.
    pub fn parse(
        raw: &str,
        variant: ModelVariant,
        tokens: usize,
        duration: std::time::Duration,
    ) -> AgentResponse {
        let (thinking, remainder) = Self::extract_thinking(raw);
        let (tool_calls, content) = Self::extract_tool_calls(&remainder);

        let tps = tokens as f64 / duration.as_secs_f64();

        AgentResponse {
            thinking,
            content: content.trim().to_string(),
            tool_calls,
            tokens_generated: tokens,
            tokens_per_second: tps,
            model_variant: variant,
        }
    }

    /// Extracts content within <|channel>thought ... <channel|>
    fn extract_thinking(s: &str) -> (Option<String>, String) {
        if let Some(start_idx) = s.find("<|channel>thought") {
            let inner = &s[start_idx + 17..];
            if let Some(end_idx) = inner.find("<channel|>") {
                let thinking = inner[..end_idx].trim().to_string();
                let remainder = s[..start_idx].to_string() + &inner[end_idx + 10..];
                return (Some(thinking), remainder);
            }
        }
        (None, s.to_string())
    }

    /// Extracts <|tool_call>call:name{args}<tool_call|> blocks
    fn extract_tool_calls(s: &str) -> (Vec<ToolCall>, String) {
        let mut tool_calls = Vec::new();
        let mut remainder = s.to_string();
        
        // Simple regex-like parsing for tool call blocks
        while let Some(start_idx) = remainder.find("<|tool_call>call:") {
            let sub = &remainder[start_idx + 17..];
            if let Some(end_idx) = sub.find("<tool_call|>") {
                let call_str = &sub[..end_idx];
                
                // Parse "name{args}"
                if let Some(brace_idx) = call_str.find('{') {
                    let name = &call_str[..brace_idx];
                    let args_str = &call_str[brace_idx..];
                    
                    if let Ok(args) = serde_json::from_str(args_str) {
                        tool_calls.push(ToolCall {
                            name: name.to_string(),
                            arguments: args,
                        });
                    }
                }
                
                // Remove the tool call from the text
                let before = &remainder[..start_idx];
                let after = &sub[end_idx + 12..];
                remainder = before.to_string() + after;
            } else {
                break;
            }
        }
        
        (tool_calls, remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_selector::ModelVariant;
    use std::time::Duration;

    #[test]
    fn test_parse_thinking() {
        let raw = "<|channel>thought\nI need to search for weather.<channel|>\nSure, let me help.";
        let (thinking, remainder) = ResponseParser::extract_thinking(raw);
        assert_eq!(thinking, Some("I need to search for weather.".to_string()));
        assert_eq!(remainder, "\nSure, let me help.");
    }

    #[test]
    fn test_parse_tool_calls() {
        let raw = "Searching...<|tool_call>call:get_weather{\"city\":\"London\"}<tool_call|>";
        let (calls, remainder) = ResponseParser::extract_tool_calls(raw);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].arguments["city"], "London");
        assert_eq!(remainder, "Searching...");
    }

    #[test]
    fn test_parse_full_response() {
        let raw = "<|channel>thought\nReasoning...<channel|>\nWait, I should call a tool.<|tool_call>call:calculator{\"op\":\"add\",\"args\":[5,5]}<tool_call|>";
        let resp = ResponseParser::parse(raw, ModelVariant::E4bQ4, 10, Duration::from_secs(1));
        assert_eq!(resp.thinking, Some("Reasoning...".to_string()));
        assert_eq!(resp.content, "Wait, I should call a tool.");
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "calculator");
    }
}
