/// ECS Components representing the configuration and state of Workflow Nodes.
///
/// Each struct here roughly corresponds to a "Node Type" in the visual graph editor.
/// These components are attached to Entities and processed by Systems in `src/systems/`.
pub mod agent;
pub mod control;
pub mod core;
pub mod execution_state;
pub mod integration;
pub mod io;
pub mod logic;
pub mod manipulation;
pub mod observability;
pub mod pipeline;
pub mod schema;
pub mod security;
pub mod shadow;

// Explicit re-exports for better maintainability and discovery
pub use self::agent::{
    AgentConfig, GenerationSettings, HistoryConfig, OutputMode, ToolChoice, ToolDefinition,
};
pub use self::control::CheckpointConfig;
pub use self::core::{Edge, EdgeLabel, Inbox, NodeConfig, Outbox, PinnedOutput};
pub use self::execution_state::{ActiveWorkflowState, DataRef};
pub use self::integration::{IntegrationConfig, PayloadMapper};
pub use self::io::{CronConfig, Frequency, HttpConfig, WebhookConfig};
pub use self::logic::{ScriptConfig, SwitchConfig};
pub use self::manipulation::{
    AggregateConfig, BatchState, ExpressionConfig, SplitConfig, StatsConfig, TransformConfig,
    WindowConfig, WindowOp, WindowState,
};
pub use self::observability::{Sensitive, Trace, TraceInput, TraceNode, TraceStart};
pub use self::pipeline::{ExecutionContext, ExecutionResult, PipelineNode, ReadyToExecute};
pub use self::schema::{ExpectedOutput, Requirements};
pub use self::security::{AuthConfig, SecretConfig};
pub use self::shadow::{MockConfig, ShadowExecution};

// Re-export common resources
pub use crate::resources::{AgentConcurrency, WorkDone};
