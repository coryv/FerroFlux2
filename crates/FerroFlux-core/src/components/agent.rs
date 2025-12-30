use bevy_ecs::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use serde_json::Value;

/// Defines the structural contract for the Agent's output.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub enum OutputMode {
    /// Free-form conversational text.
    #[default]
    Text,
    /// Strict JSON output adhering to a predefined structural schema.
    JsonStrict,
    /// JSON output adhering to a custom JSON Schema provided at runtime.
    JsonSchema(Value),
}

/// Defines a tool that the agent can invoke.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema for arguments
}

/// Governance policy for tool selection by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub enum ToolChoice {
    /// (Default) The model autonomously decides whether to use a tool.
    #[default]
    Auto,
    /// Forbids the model from using any tools.
    None,
    /// Forces the model to use at least one tool.
    Required,
    /// Forces the model to use a specific named tool.
    Specific(String),
}

/// Configuration for message history.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub window_size: usize,
    pub session_id_key: String,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            window_size: 10,
            session_id_key: "session_id".to_string(),
        }
    }
}

/// Configuration for generation settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct GenerationSettings {
    pub temperature: f32, // Default 0.7
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
}

/// Configuration for an AI Agent Node.
///
/// Agents are the autonomous "workers" in a flow, capable of reasoning,
/// using tools, and maintaining conversation history.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct AgentConfig {
    /// The LLM provider (e.g., "openai", "anthropic").
    pub provider: String,
    /// The specific model identifier (e.g., "gpt-4-turbo").
    pub model: String,
    /// The system prompt aka "Role" definition.
    #[serde(default = "default_system_instruction")]
    pub system_instruction: String,
    /// Template for the user message, supporting variable interpolation.
    #[serde(default = "default_user_prompt_template")]
    pub user_prompt_template: String,
    /// Tunable parameters for the generation stochasticity.
    #[serde(default)]
    pub generation_settings: GenerationSettings,
    /// Format constraints for the response.
    #[serde(default)]
    pub output_mode: OutputMode,
    /// Memory management settings.
    #[serde(default)]
    pub history_config: HistoryConfig,
    /// Capabilities available to this agent.
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    /// Policy for tool usage.
    #[serde(default)]
    pub tool_choice: ToolChoice,
    /// Optional key to map the final answer to in the output structure.
    #[serde(default)]
    pub result_key: Option<String>,
    /// Optional slug reference to a secure connection (SecretStore).
    /// If present, this takes precedence over `provider.env_var` lookups.
    #[serde(default)]
    pub connection_slug: Option<String>,
}

fn default_system_instruction() -> String {
    "You are a helpful assistant.".to_string()
}

fn default_user_prompt_template() -> String {
    "{{user_prompt}}".to_string()
}
