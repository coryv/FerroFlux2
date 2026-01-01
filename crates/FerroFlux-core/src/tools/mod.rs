use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

pub mod primitives;
pub mod registry;
mod tests;

/// Context provided to a Tool during execution.
///
/// This context holds the ephemeral state of the current node execution,
/// allowing tools to read inputs and write outputs.
pub struct ToolContext<'a> {
    /// Variables local to the current node execution pipeline.
    pub local: &'a mut HashMap<String, Value>,
    /// global workflow memory (read/write).
    pub memory: &'a mut HashMap<String, Value>,
    /// Correlation ID for the execution flow.
    pub trace_id: String,
    /// System event bus for emitting telemetry.
    pub event_bus: Option<crate::api::events::SystemEventBus>,
    /// Whether the current execution is a safe simulation ("Shadow Mode").
    pub shadow_mode: bool,
    /// Mock configurations for specific tools when in Shadow Mode.
    pub shadow_masks: &'a HashMap<String, crate::components::shadow::MockConfig>,
}

/// A "Tool" is an atomic unit of logic.
///
/// Tools are stateless and re-entrant. They take a configuration (params)
/// and a context, perform an action, and return a result.
pub trait Tool: Send + Sync {
    /// The unique identifier for this tool (e.g., "http_client", "switch").
    fn id(&self) -> &'static str;

    /// Executes the tool's logic.
    ///
    /// # Arguments
    /// * `context` - Mutable access to the node's execution context.
    /// * `params` - The resolved configuration for this step (variables already interpolated).
    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value>;
}

/// Helper to register all core primitive tools.
pub fn register_core_tools(registry: &mut registry::ToolRegistry) {
    use primitives::*;
    registry.register(HttpClientTool);
    registry.register(SwitchTool);
    registry.register(JsonQueryTool);
    registry.register(EmitTool);
    registry.register(LogicTool);
    registry.register(LogTool);
    registry.register(SleepTool);
    registry.register(SetVarTool);
    registry.register(GetVarTool);
    registry.register(MathTool);
    registry.register(RhaiTool::default());
    registry.register(TraceTool);
}
