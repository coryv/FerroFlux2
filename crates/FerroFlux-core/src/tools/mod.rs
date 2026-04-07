pub use ferroflux_types::tool::{SecretResolver, Tool, ToolRegistry};

/// Re-export DataRef from types for convenience within tools.
pub use ferroflux_types::data_ref::DataRef;

/// Re-export the portable ToolContext.
pub use ferroflux_types::tool::ToolContext;

use ferroflux_db::secrets::SecretStore;

/// A wrapper that implements `SecretResolver` for the core runtime.
///
/// It bridges the synchronous `resolve_connection` call required by tools
/// to the asynchronous `DatabaseSecretStore` using the provided Tokio runtime.
pub struct CoreSecretResolver<'a> {
    pub tenant_id: ferroflux_types::tenant::TenantId,
    pub store: &'a crate::secrets::DatabaseSecretStore,
    pub runtime: &'a ferroflux_types::resources::TokioRuntime,
    pub refresh_locks: Option<&'a ferroflux_db::oauth2::TokenRefreshLocks>,
}

impl<'a> SecretResolver for CoreSecretResolver<'a> {
    fn resolve_connection(
        &self,
        tenant: &ferroflux_types::tenant::TenantId,
        slug: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let conn_data = self
            .runtime
            .0
            .block_on(async { self.store.resolve_connection(tenant, slug).await })?;

        // Handle OAuth2 Refresh if needed
        if let Some(auth_type) = conn_data.get("auth_type").and_then(|v| v.as_str())
            && auth_type == "OAuth2"
        {
            let refreshed_token = ferroflux_db::oauth2::resolve_oauth2_token(
                tenant,
                slug,
                &conn_data,
                self.store,
                self.runtime,
                self.refresh_locks,
            )?;

            // Patch the conn_data with the fresh token so the tool sees it
            let mut patched = conn_data.clone();
            if let Some(obj) = patched.as_object_mut() {
                obj.insert(
                    "access_token".to_string(),
                    serde_json::Value::String(refreshed_token),
                );
            }
            return Ok(patched);
        }

        Ok(conn_data)
    }

    fn get_secret(
        &self,
        tenant: &ferroflux_types::tenant::TenantId,
        key: &str,
    ) -> anyhow::Result<String> {
        self.runtime
            .0
            .block_on(async { self.store.get_secret(tenant, key).await })
    }
}

/// Implementation of `ActionExecutor` for the core engine.
pub struct CoreActionExecutor<'a> {
    pub definitions: &'a crate::resources::DefinitionRegistry,
    pub tools: &'a ferroflux_types::tool::ToolRegistry,
    pub event_bus: Option<crate::api::events::SystemEventBus>,
    pub store: Option<&'a crate::store::BlobStore>,
    pub secret_store: Option<&'a crate::secrets::DatabaseSecretStore>,
    pub runtime: &'a ferroflux_types::resources::TokioRuntime,
    pub refresh_locks: Option<&'a ferroflux_db::oauth2::TokenRefreshLocks>,
    pub workflow_config: HashMap<String, Value>,
}

impl<'a> ferroflux_types::tool::ActionExecutor for CoreActionExecutor<'a> {
    fn execute(
        &self,
        _tenant_id: &ferroflux_types::tenant::TenantId,
        action_id: &str,
        params: Value,
        context: &mut ToolContext,
    ) -> anyhow::Result<Value> {
        use crate::components::pipeline::PipelineNode;
        use crate::components::execution_state::ActiveWorkflowState;

        let mut node = PipelineNode::new(
            action_id.to_string(),
            params.as_object().cloned().unwrap_or_default().into_iter().collect(),
        );

        let mut workflow_state = ActiveWorkflowState::default();
        // Propagate current context to the sub-action if needed
        // For 'Call', we often start fresh or pass explicit inputs.

        let emissions = crate::systems::pipeline::execution::execute_pipeline_node(
            &mut node,
            &mut workflow_state,
            self.definitions,
            self.tools,
            context.memory,
            context.trace_id.clone(),
            self.event_bus.clone(),
            self.store,
            None, // No shadow mode for sub-calls yet
            None,
            self.secret_store,
            Some(self.runtime),
            self.refresh_locks,
            self.workflow_config.clone(),
        )?;

        // Return the first value from any numeric/string port, or the whole Success object
        for (port, val, _) in emissions {
            if port != "_next" {
                return Ok(val);
            }
        }

        Ok(Value::Null)
    }
}

use serde_json::Value;
use std::collections::HashMap;
