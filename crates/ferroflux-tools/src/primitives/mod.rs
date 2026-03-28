pub mod aws_sigv4;
pub mod emit;

pub mod http_client;
pub mod json_query;
pub mod log;
pub mod logic;
pub mod math;
pub mod mongo_query;
pub mod paginate;
pub mod redis_query;
pub mod request;
pub mod rhai;
pub mod sleep;
pub mod smtp;
pub mod sql_query;
pub mod stats;


pub mod trace;
pub mod utils;
pub mod variable;
pub mod verify_signature;

pub use self::emit::EmitTool;
pub use self::http_client::HttpClientTool;
pub use self::json_query::JsonQueryTool;
pub use self::log::LogTool;
pub use self::logic::LogicTool;
pub use self::math::MathTool;
pub use self::paginate::PaginateTool;
pub use self::rhai::RhaiTool;
pub use self::sleep::SleepTool;
pub use self::smtp::SmtpTool;
pub use self::stats::StatsTool;

pub use self::trace::TraceTool;
pub use self::variable::{GetVarTool, SetVarTool};
pub use self::verify_signature::VerifySignatureTool;
pub use self::sql_query::SqlTool;
pub use self::mongo_query::MongoTool;
pub use self::redis_query::RedisTool;


use ferroflux_db::oauth2::TokenRefreshLocks;
use ferroflux_db::secrets::{DatabaseSecretStore, SecretStore};
use ferroflux_types::resources::TokioRuntime;
use ferroflux_types::tenant::TenantId;
use ferroflux_types::tool::SecretResolver;

/// A wrapper that implements `SecretResolver` for the core runtime.
pub struct CoreSecretResolver<'a> {
    pub tenant_id: TenantId,
    pub store: &'a DatabaseSecretStore,
    pub runtime: &'a TokioRuntime,
    pub refresh_locks: Option<&'a TokenRefreshLocks>,
}

impl<'a> SecretResolver for CoreSecretResolver<'a> {
    fn resolve_connection(&self, tenant: &TenantId, slug: &str) -> anyhow::Result<serde_json::Value> {
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

    fn get_secret(&self, tenant: &TenantId, key: &str) -> anyhow::Result<String> {
        self.runtime
            .0
            .block_on(async { self.store.get_secret(tenant, key).await })
    }
}
