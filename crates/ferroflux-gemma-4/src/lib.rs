pub mod agent;
pub mod config;
pub mod device;
pub mod model_selector;
pub mod parse;
pub mod syscheck;
pub mod template;
pub mod types;

pub use agent::GemmaAgent;
pub use config::AgentConfig;
pub use device::{select_device, select_dtype};
pub use model_selector::{get_profile, select_model, ModelProfile, ModelVariant};
pub use syscheck::SystemResources;
pub use types::{AgentResponse, ChatMessage, ChatRole, ToolCall, ToolDefinition, ToolResponse};

/// A convenience function to initialize the default agent for the current system.
pub fn load_default_agent() -> anyhow::Result<GemmaAgent> {
    GemmaAgent::load(AgentConfig::default())
}
