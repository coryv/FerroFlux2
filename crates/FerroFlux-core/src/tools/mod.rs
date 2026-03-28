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
