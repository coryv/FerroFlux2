use crate::blob::BlobStore;
use crate::events::SystemEventBus;
use crate::shadow::MockConfig;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// A reference to data — inline JSON or a BlobStore ticket.
pub use crate::data_ref::DataRef;

/// Context provided to a [`Tool`] during execution.
///
/// This holds the ephemeral state of the current node execution: inputs,
/// outputs, secrets access, the event bus, and shadow-mode configuration.
pub struct ToolContext<'a> {
    /// Variables local to the current node execution pipeline.
    pub local: &'a mut HashMap<String, DataRef>,
    /// Global workflow memory (read/write across nodes).
    pub memory: &'a mut HashMap<String, Value>,
    /// Correlation ID for the execution flow.
    pub trace_id: String,
    /// System event bus for emitting telemetry.
    pub event_bus: Option<SystemEventBus>,
    /// Whether the current execution is a safe simulation ("Shadow Mode").
    pub shadow_mode: bool,
    /// Mock configurations for specific tools when in Shadow Mode.
    pub shadow_masks: &'a HashMap<String, MockConfig>,
    /// Access to BlobStore for resolving DataRefs.
    pub store: Option<&'a BlobStore>,
    /// Resolver for connections and secrets.
    pub secrets: Option<&'a dyn SecretResolver>,
}

/// Abstract interface for resolving secrets and connections.
///
/// Allows portable tools to request credentials without knowing if they
/// come from a database, environment, or vault.
pub trait SecretResolver: Send + Sync {
    /// Resolve a connection reference (slug) to the full credential object.
    fn resolve_connection(
        &self,
        tenant: &crate::tenant::TenantId,
        slug: &str,
    ) -> Result<serde_json::Value>;

    /// Retrieve a plain secret string by key.
    fn get_secret(&self, tenant: &crate::tenant::TenantId, key: &str) -> Result<String>;
}

/// A stateless, re-entrant unit of logic executed as part of a pipeline step.
///
/// Tools are registered in the `ToolRegistry` and invoked by the pipeline
/// execution system. They receive configuration via `params` (already with
/// variables interpolated) and can read/write the execution `context`
/// (which in the core runtime is the fuller `ToolContext` with secrets access).
pub trait Tool: Send + Sync {
    /// The unique identifier for this tool, e.g. `"http_client"`, `"switch"`.
    fn id(&self) -> &'static str;

    /// Executes the tool's logic and returns its output value.
    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value>;
}

/// Runtime registry of all available [`Tool`] implementations.
///
/// Populated at startup by `register_core_tools()` and optionally by
/// platform-specific registrations. Stored as a Bevy ECS resource.
#[derive(bevy_ecs::prelude::Resource, Default, Clone)]
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, std::sync::Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Registers a new tool implementation.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let id = tool.id().to_string();
        self.tools
            .insert(id.clone(), std::sync::Arc::new(tool));
        tracing::trace!("Registered tool: {}", id);
    }

    /// Retrieves a tool by its unique ID.
    pub fn get(&self, id: &str) -> Option<std::sync::Arc<dyn Tool>> {
        self.tools.get(id).cloned()
    }

    /// Lists all registered tool IDs.
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}
