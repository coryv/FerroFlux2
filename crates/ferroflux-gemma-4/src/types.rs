use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Model,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_responses: Option<Vec<ToolResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub thinking: Option<String>,      // Extracted from <|channel>thought
    pub content: String,               // Final answer text
    pub tool_calls: Vec<ToolCall>,     // Parsed tool invocations
    pub tokens_generated: usize,
    pub tokens_per_second: f64,
    pub model_variant: crate::model_selector::ModelVariant,
}

impl ChatMessage {
    pub fn new_user(content: String) -> Self {
        Self {
            role: ChatRole::User,
            content,
            tool_calls: None,
            tool_responses: None,
        }
    }

    pub fn new_system(content: String) -> Self {
        Self {
            role: ChatRole::System,
            content,
            tool_calls: None,
            tool_responses: None,
        }
    }

    pub fn new_model(content: String) -> Self {
        Self {
            role: ChatRole::Model,
            content,
            tool_calls: None,
            tool_responses: None,
        }
    }
}
