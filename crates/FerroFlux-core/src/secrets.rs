use crate::domain::TenantId;
use crate::store::database::PersistentStore;
use anyhow::{Context, Result};
use async_trait::async_trait;
use bevy_ecs::system::Resource;
use serde_json::Value;
use std::env;

/// Trait for retrieving secrets, abstracting the source (Env, Vault, DB, etc.)
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Retrieve a secret by key, scoped to a tenant.
    async fn get_secret(&self, tenant: &TenantId, key: &str) -> Result<String>;

    /// Resolve a connection reference (slug) to the full credential object.
    async fn resolve_connection(&self, tenant: &TenantId, slug: &str) -> Result<Value>;
}

/// Implementation that reads from environment variables (Legacy/Dev mode).
#[derive(Clone, Resource)]
pub struct EnvSecretStore;

#[async_trait]
impl SecretStore for EnvSecretStore {
    async fn get_secret(&self, _tenant: &TenantId, key: &str) -> Result<String> {
        env::var(key).map_err(|_| anyhow::anyhow!("Secret '{}' not found in environment", key))
    }

    async fn resolve_connection(&self, _tenant: &TenantId, slug: &str) -> Result<Value> {
        Err(anyhow::anyhow!(
            "EnvSecretStore cannot resolve connection '{}'.",
            slug
        ))
    }
}

/// Implementation that reads encrypted connections from the database.
///
/// ## Security
/// - Retrieves encrypted blobs from the `connections` table.
/// - Decrypts them using the `master_key` and the stored `nonce`.
#[derive(Clone, Resource)]
pub struct DatabaseSecretStore {
    store: PersistentStore,
    master_key: Vec<u8>,
}

impl DatabaseSecretStore {
    pub fn new(store: PersistentStore, master_key: Vec<u8>) -> Self {
        Self { store, master_key }
    }
}

#[async_trait]
impl SecretStore for DatabaseSecretStore {
    async fn get_secret(&self, _tenant: &TenantId, key: &str) -> Result<String> {
        // Fallback to env for single values for now.
        env::var(key)
            .map_err(|_| anyhow::anyhow!("Secret '{}' not found in environment (DB fallback)", key))
    }

    async fn resolve_connection(&self, tenant: &TenantId, slug: &str) -> Result<Value> {
        // Fully async execution (no block_on needed)
        let (_pt, enc_data, nonce, _, _) =
            self.store
                .get_connection_by_slug(tenant, slug)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Connection '{}' not found", slug))?;

        let decrypted = crate::security::encryption::decrypt(&enc_data, &self.master_key, &nonce)
            .context("Decryption failed")?;

        let json: Value =
            serde_json::from_slice(&decrypted).context("Invalid JSON in connection data")?;

        Ok(json)
    }
}
