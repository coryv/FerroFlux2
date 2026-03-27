use anyhow::{Context, Result};
use async_trait::async_trait;
use bevy_ecs::system::Resource;
use ferroflux_types::tenant::TenantId;
use serde_json::Value;
use std::env;

use crate::workflow::PersistentStore;

/// Trait for retrieving secrets, abstracting the underlying source.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Retrieve a secret value by key, scoped to a tenant.
    async fn get_secret(&self, tenant: &TenantId, key: &str) -> Result<String>;

    /// Resolve a connection slug to the full decrypted credential object.
    async fn resolve_connection(&self, tenant: &TenantId, slug: &str) -> Result<Value>;
}

/// Secret store backed by the database — credentials are stored AES-GCM encrypted.
///
/// ## Security
/// - Fetches encrypted blobs from the `connections` table.
/// - Decrypts them with the `master_key` and the stored `nonce`.
/// - The master key never leaves this struct; callers get plaintext JSON.
#[derive(Clone, Resource)]
pub struct DatabaseSecretStore {
    store: PersistentStore,
    master_key: Vec<u8>,
}

impl DatabaseSecretStore {
    pub fn new(store: PersistentStore, master_key: Vec<u8>) -> Self {
        Self { store, master_key }
    }

    /// Encrypts and persists updated connection data (e.g., after OAuth2 token refresh).
    pub async fn update_connection_data(
        &self,
        tenant: &TenantId,
        slug: &str,
        updated_json: &[u8],
    ) -> Result<()> {
        let (enc_data, nonce) =
            ferroflux_security::encryption::encrypt(updated_json, &self.master_key)
                .context("Encryption failed during connection data update")?;
        self.store
            .update_connection_encrypted_data(tenant, slug, &enc_data, &nonce)
            .await
    }
}

#[async_trait]
impl SecretStore for DatabaseSecretStore {
    async fn get_secret(&self, _tenant: &TenantId, key: &str) -> Result<String> {
        env::var(key)
            .map_err(|_| anyhow::anyhow!("Secret '{}' not found in environment", key))
    }

    async fn resolve_connection(&self, tenant: &TenantId, slug: &str) -> Result<Value> {
        let (_pt, enc_data, nonce, _, _) = self
            .store
            .get_connection_by_slug(tenant, slug)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Connection '{}' not found", slug))?;

        let decrypted =
            ferroflux_security::encryption::decrypt(&enc_data, &self.master_key, &nonce)
                .context("Decryption failed")?;

        let json: Value =
            serde_json::from_slice(&decrypted).context("Invalid JSON in connection data")?;

        Ok(json)
    }
}
