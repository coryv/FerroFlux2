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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::PersistentStore;

    const MASTER_KEY: [u8; 32] = [0xAB; 32];

    async fn in_memory_store() -> PersistentStore {
        PersistentStore::new("sqlite::memory:").await.unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::from("test_tenant")
    }

    async fn store_with_connection(
        store: &PersistentStore,
        tenant: &TenantId,
        slug: &str,
        creds: &Value,
    ) {
        let plaintext = serde_json::to_vec(creds).unwrap();
        let (enc_data, nonce) =
            ferroflux_security::encryption::encrypt(&plaintext, &MASTER_KEY).unwrap();
        store
            .save_connection(tenant, slug, "Test Conn", "oauth2", &enc_data, &nonce, "active")
            .await
            .unwrap();
    }

    // ── resolve_connection round-trip ────────────────────────────────────────

    #[tokio::test]
    async fn resolve_connection_decrypts_stored_credentials() {
        let store = in_memory_store().await;
        let tenant = tenant();
        let creds = serde_json::json!({ "access_token": "tok_abc", "refresh_token": "ref_xyz" });
        store_with_connection(&store, &tenant, "github", &creds).await;

        let secret_store = DatabaseSecretStore::new(store, MASTER_KEY.to_vec());
        let resolved = secret_store.resolve_connection(&tenant, "github").await.unwrap();

        assert_eq!(resolved["access_token"], "tok_abc");
        assert_eq!(resolved["refresh_token"], "ref_xyz");
    }

    #[tokio::test]
    async fn resolve_connection_missing_returns_error() {
        let store = in_memory_store().await;
        let secret_store = DatabaseSecretStore::new(store, MASTER_KEY.to_vec());

        let err = secret_store
            .resolve_connection(&tenant(), "nonexistent")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not found"), "{err}");
    }

    #[tokio::test]
    async fn resolve_connection_tenant_isolated() {
        let store = in_memory_store().await;
        let tenant_a = tenant();
        let tenant_b = TenantId::from("other_tenant");
        let creds = serde_json::json!({ "token": "secret_a" });
        store_with_connection(&store, &tenant_a, "stripe", &creds).await;

        let secret_store = DatabaseSecretStore::new(store, MASTER_KEY.to_vec());
        // tenant_b cannot see tenant_a's connection
        let err = secret_store
            .resolve_connection(&tenant_b, "stripe")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not found"), "{err}");
    }

    // ── update_connection_data re-encrypts and resolves correctly ─────────────

    #[tokio::test]
    async fn update_connection_data_refreshed_token_resolves() {
        let store = in_memory_store().await;
        let tenant = tenant();
        let original = serde_json::json!({ "access_token": "old_token" });
        store_with_connection(&store, &tenant, "slack", &original).await;

        let secret_store = DatabaseSecretStore::new(store, MASTER_KEY.to_vec());

        let updated = serde_json::json!({ "access_token": "new_token" });
        let updated_bytes = serde_json::to_vec(&updated).unwrap();
        secret_store
            .update_connection_data(&tenant, "slack", &updated_bytes)
            .await
            .unwrap();

        let resolved = secret_store.resolve_connection(&tenant, "slack").await.unwrap();
        assert_eq!(resolved["access_token"], "new_token");
    }

    // ── get_secret reads from environment ─────────────────────────────────────

    #[tokio::test]
    async fn get_secret_reads_env_var() {
        let store = in_memory_store().await;
        let secret_store = DatabaseSecretStore::new(store, MASTER_KEY.to_vec());

        unsafe { std::env::set_var("TEST_FF_SECRET_KEY", "super_secret_value"); }
        let val = secret_store.get_secret(&tenant(), "TEST_FF_SECRET_KEY").await.unwrap();
        assert_eq!(val, "super_secret_value");
        unsafe { std::env::remove_var("TEST_FF_SECRET_KEY"); }
    }

    #[tokio::test]
    async fn get_secret_missing_env_var_returns_error() {
        let store = in_memory_store().await;
        let secret_store = DatabaseSecretStore::new(store, MASTER_KEY.to_vec());

        unsafe { std::env::remove_var("FF_THIS_DOES_NOT_EXIST"); }
        let err = secret_store
            .get_secret(&tenant(), "FF_THIS_DOES_NOT_EXIST")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not found"), "{err}");
    }

    // ── wrong key cannot decrypt ──────────────────────────────────────────────

    #[tokio::test]
    async fn resolve_connection_wrong_key_returns_error() {
        let store = in_memory_store().await;
        let tenant = tenant();
        let creds = serde_json::json!({ "token": "secret" });
        store_with_connection(&store, &tenant, "github", &creds).await;

        // Different key — decryption must fail
        let wrong_key = [0x00u8; 32];
        let secret_store = DatabaseSecretStore::new(store, wrong_key.to_vec());
        let err = secret_store.resolve_connection(&tenant, "github").await.unwrap_err();
        assert!(
            err.to_string().contains("Decryption") || err.to_string().contains("decrypt"),
            "{err}"
        );
    }
}
