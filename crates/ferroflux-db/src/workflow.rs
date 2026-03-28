use anyhow::Result;
use bevy_ecs::prelude::*;
use ferroflux_types::tenant::TenantId;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Row, Sqlite};

/// SQLite persistence layer for workflows, connections, and checkpoints.
///
/// Every table includes a `tenant_id` column enforcing logical multi-tenancy.
#[derive(Clone, Debug, Resource)]
pub struct PersistentStore {
    pool: Pool<Sqlite>,
}

impl PersistentStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        let connection_options = crate::sqlite_options_from_url(db_url)?;

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        // Run base migrations
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                blueprint_json TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                status TEXT DEFAULT 'active',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS checkpoints (
                token TEXT PRIMARY KEY,
                node_id TEXT,
                data BLOB,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                tenant_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                slug TEXT NOT NULL,
                name TEXT,
                provider_type TEXT,
                encrypted_data BLOB,
                nonce BLOB,
                status TEXT DEFAULT 'unverified',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(tenant_id, slug)
            );
            "#,
        )
        .execute(&pool)
        .await?;

        // Additive column migrations (idempotent – error means column already exists)
        let _ = sqlx::query("ALTER TABLE workflows ADD COLUMN tenant_id TEXT DEFAULT 'default_tenant'")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE checkpoints ADD COLUMN tenant_id TEXT DEFAULT 'default_tenant'")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE workflows ADD COLUMN status TEXT DEFAULT 'active'")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE connections ADD COLUMN name TEXT")
            .execute(&pool)
            .await;
        let _ = sqlx::query("ALTER TABLE connections ADD COLUMN status TEXT DEFAULT 'unverified'")
            .execute(&pool)
            .await;
        let _ = sqlx::query(
            "ALTER TABLE connections ADD COLUMN updated_at DATETIME DEFAULT CURRENT_TIMESTAMP",
        )
        .execute(&pool)
        .await;

        Ok(Self { pool })
    }

    // ── Workflows ─────────────────────────────────────────────────────────────

    pub async fn save_workflow(
        &self,
        tenant: &TenantId,
        id: &str,
        name: &str,
        description: Option<&str>,
        json: &str,
        status: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO workflows (id, name, description, blueprint_json, tenant_id, status)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                blueprint_json = excluded.blueprint_json,
                tenant_id = excluded.tenant_id,
                status = excluded.status,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(json)
        .bind(tenant.as_ref())
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_active_workflows(
        &self,
        tenant: &TenantId,
    ) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query(
            "SELECT id, blueprint_json, status FROM workflows WHERE tenant_id = ? AND status = 'active'",
        )
        .bind(tenant.as_ref())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let json: String = row.get("blueprint_json");
                let status: String = row.try_get("status").unwrap_or("active".into());
                (id, json, status)
            })
            .collect())
    }

    pub async fn list_workflows(
        &self,
        tenant: &TenantId,
    ) -> Result<Vec<(String, String, Option<String>, String, String)>> {
        let rows = sqlx::query(
            "SELECT id, name, description, status, updated_at FROM workflows WHERE tenant_id = ? ORDER BY updated_at DESC",
        )
        .bind(tenant.as_ref())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: String = row.get("id");
                let name: String = row.get("name");
                let description: Option<String> = row.get("description");
                let status: String = row.try_get("status").unwrap_or("active".into());
                let updated_at: String = row.try_get("updated_at").unwrap_or_default();
                (id, name, description, status, updated_at)
            })
            .collect())
    }

    pub async fn get_workflow(
        &self,
        tenant: &TenantId,
        id: &str,
    ) -> Result<Option<(String, String, Option<String>, String, String)>> {
        let row = sqlx::query(
            "SELECT id, name, description, blueprint_json, status FROM workflows WHERE tenant_id = ? AND id = ?",
        )
        .bind(tenant.as_ref())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let json: String = row.get("blueprint_json");
            let status: String = row.try_get("status").unwrap_or("active".into());
            (id, name, description, json, status)
        }))
    }

    pub async fn delete_workflow(&self, tenant: &TenantId, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM workflows WHERE tenant_id = ? AND id = ?")
            .bind(tenant.as_ref())
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── Checkpoints ───────────────────────────────────────────────────────────

    pub async fn save_checkpoint(
        &self,
        tenant: &TenantId,
        token: &str,
        node_id: uuid::Uuid,
        data: &[u8],
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let metadata_json = serde_json::to_string(metadata)?;
        let node_id_str = node_id.to_string();
        sqlx::query(
            "INSERT INTO checkpoints (token, node_id, data, metadata, tenant_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(token)
        .bind(node_id_str)
        .bind(data)
        .bind(metadata_json)
        .bind(tenant.as_ref())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn claim_checkpoint(
        &self,
        tenant: &TenantId,
        token: &str,
    ) -> Result<Option<(uuid::Uuid, Vec<u8>, std::collections::HashMap<String, String>)>> {
        let row = sqlx::query(
            "SELECT node_id, data, metadata FROM checkpoints WHERE token = ? AND tenant_id = ?",
        )
        .bind(token)
        .bind(tenant.as_ref())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let node_id_str: String = row.get("node_id");
            let data: Vec<u8> = row.get("data");
            let metadata_str: String = row.get("metadata");
            let node_id = uuid::Uuid::parse_str(&node_id_str)?;
            let metadata: std::collections::HashMap<String, String> =
                serde_json::from_str(&metadata_str)?;

            // Consume-on-read
            sqlx::query("DELETE FROM checkpoints WHERE token = ? AND tenant_id = ?")
                .bind(token)
                .bind(tenant.as_ref())
                .execute(&self.pool)
                .await?;

            Ok(Some((node_id, data, metadata)))
        } else {
            Ok(None)
        }
    }

    // ── Connections ───────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub async fn save_connection(
        &self,
        tenant: &TenantId,
        slug: &str,
        name: &str,
        provider_type: &str,
        encrypted_data: &[u8],
        nonce: &[u8],
        status: &str,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO connections (id, tenant_id, slug, name, provider_type, encrypted_data, nonce, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(tenant_id, slug) DO UPDATE SET
                name = excluded.name,
                provider_type = excluded.provider_type,
                encrypted_data = excluded.encrypted_data,
                nonce = excluded.nonce,
                status = excluded.status,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id)
        .bind(tenant.as_ref())
        .bind(slug)
        .bind(name)
        .bind(provider_type)
        .bind(encrypted_data)
        .bind(nonce)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_connection_by_slug(
        &self,
        tenant: &TenantId,
        slug: &str,
    ) -> Result<Option<(String, Vec<u8>, Vec<u8>, String, String)>> {
        let row = sqlx::query(
            "SELECT provider_type, encrypted_data, nonce, name, status FROM connections WHERE tenant_id = ? AND slug = ?",
        )
        .bind(tenant.as_ref())
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let provider_type: String = row.get("provider_type");
            let data: Vec<u8> = row.get("encrypted_data");
            let nonce: Vec<u8> = row.get("nonce");
            let name: String = row.try_get("name").unwrap_or_else(|_| slug.to_string());
            let status: String = row.try_get("status").unwrap_or("unverified".into());
            (provider_type, data, nonce, name, status)
        }))
    }

    pub async fn mark_connection_status(
        &self,
        tenant: &TenantId,
        slug: &str,
        status: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE connections SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE tenant_id = ? AND slug = ?",
        )
        .bind(status)
        .bind(tenant.as_ref())
        .bind(slug)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_connections(
        &self,
        tenant: &TenantId,
    ) -> Result<Vec<(String, String, String, String, i64, String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT
                c.slug, c.name, c.provider_type, c.status, c.created_at, c.updated_at,
                (SELECT COUNT(*) FROM workflows w WHERE w.tenant_id = c.tenant_id AND w.blueprint_json LIKE '%' || c.slug || '%') as usage_count
            FROM connections c
            WHERE c.tenant_id = ?
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(tenant.as_ref())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let slug: String = row.get("slug");
                let name: String = row
                    .try_get("name")
                    .unwrap_or_else(|_| slug.clone());
                let provider_type: String = row.get("provider_type");
                let status: String = row
                    .try_get("status")
                    .unwrap_or("unverified".into());
                let created_at: String = row.try_get("created_at").unwrap_or_default();
                let updated_at: String = row.try_get("updated_at").unwrap_or_default();
                let usage_count: i64 = row.get("usage_count");
                (slug, name, provider_type, status, usage_count, created_at, updated_at)
            })
            .collect())
    }

    pub async fn update_connection_encrypted_data(
        &self,
        tenant: &TenantId,
        slug: &str,
        encrypted_data: &[u8],
        nonce: &[u8],
    ) -> Result<()> {
        sqlx::query(
            "UPDATE connections SET encrypted_data = ?, nonce = ?, status = 'active', updated_at = CURRENT_TIMESTAMP WHERE tenant_id = ? AND slug = ?",
        )
        .bind(encrypted_data)
        .bind(nonce)
        .bind(tenant.as_ref())
        .bind(slug)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_connection(&self, tenant: &TenantId, slug: &str) -> Result<()> {
        sqlx::query("DELETE FROM connections WHERE tenant_id = ? AND slug = ?")
            .bind(tenant.as_ref())
            .bind(slug)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn in_memory_store() -> PersistentStore {
        PersistentStore::new("sqlite::memory:").await.unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::from("test_tenant")
    }

    fn other_tenant() -> TenantId {
        TenantId::from("other_tenant")
    }

    // ── Workflow CRUD ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn save_and_load_workflow() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_workflow(&t, "wf1", "My Flow", None, r#"{"nodes":[]}"#, "active")
            .await
            .unwrap();

        let loaded = store.load_active_workflows(&t).await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "wf1");
        assert_eq!(loaded[0].1, r#"{"nodes":[]}"#);
        assert_eq!(loaded[0].2, "active");
    }

    #[tokio::test]
    async fn load_active_workflows_excludes_inactive() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_workflow(&t, "wf_active", "Active", None, "{}", "active")
            .await
            .unwrap();
        store
            .save_workflow(&t, "wf_inactive", "Inactive", None, "{}", "inactive")
            .await
            .unwrap();

        let loaded = store.load_active_workflows(&t).await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, "wf_active");
    }

    #[tokio::test]
    async fn save_workflow_upserts_on_conflict() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_workflow(&t, "wf1", "Original", None, r#"{"v":1}"#, "active")
            .await
            .unwrap();
        store
            .save_workflow(&t, "wf1", "Updated", None, r#"{"v":2}"#, "active")
            .await
            .unwrap();

        let rows = store.list_workflows(&t).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, "Updated");

        let wf = store.get_workflow(&t, "wf1").await.unwrap().unwrap();
        assert_eq!(wf.3, r#"{"v":2}"#);
    }

    #[tokio::test]
    async fn delete_workflow() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_workflow(&t, "wf1", "Flow", None, "{}", "active")
            .await
            .unwrap();
        store.delete_workflow(&t, "wf1").await.unwrap();

        let loaded = store.load_active_workflows(&t).await.unwrap();
        assert!(loaded.is_empty());
    }

    #[tokio::test]
    async fn tenant_isolation_workflows() {
        let store = in_memory_store().await;
        let t1 = tenant();
        let t2 = other_tenant();

        store
            .save_workflow(&t1, "wf_a", "A", None, "{}", "active")
            .await
            .unwrap();
        store
            .save_workflow(&t2, "wf_b", "B", None, "{}", "active")
            .await
            .unwrap();

        let t1_wfs = store.load_active_workflows(&t1).await.unwrap();
        let t2_wfs = store.load_active_workflows(&t2).await.unwrap();

        assert_eq!(t1_wfs.len(), 1);
        assert_eq!(t1_wfs[0].0, "wf_a");
        assert_eq!(t2_wfs.len(), 1);
        assert_eq!(t2_wfs[0].0, "wf_b");
    }

    // ── Checkpoints ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn save_and_claim_checkpoint() {
        let store = in_memory_store().await;
        let t = tenant();
        let node_id = uuid::Uuid::new_v4();
        let mut meta = std::collections::HashMap::new();
        meta.insert("trace_id".into(), "t42".into());

        store
            .save_checkpoint(&t, "tok1", node_id, b"checkpoint_data", &meta)
            .await
            .unwrap();

        let result = store.claim_checkpoint(&t, "tok1").await.unwrap();
        assert!(result.is_some());
        let (nid, data, m) = result.unwrap();
        assert_eq!(nid, node_id);
        assert_eq!(data, b"checkpoint_data");
        assert_eq!(m["trace_id"], "t42");
    }

    #[tokio::test]
    async fn claim_checkpoint_is_consume_on_read() {
        let store = in_memory_store().await;
        let t = tenant();
        let node_id = uuid::Uuid::new_v4();
        let meta = std::collections::HashMap::new();

        store
            .save_checkpoint(&t, "tok2", node_id, b"data", &meta)
            .await
            .unwrap();

        // First claim succeeds
        assert!(store.claim_checkpoint(&t, "tok2").await.unwrap().is_some());
        // Second claim returns None (consumed)
        assert!(store.claim_checkpoint(&t, "tok2").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn claim_checkpoint_wrong_tenant_returns_none() {
        let store = in_memory_store().await;
        let t1 = tenant();
        let t2 = other_tenant();
        let node_id = uuid::Uuid::new_v4();

        store
            .save_checkpoint(&t1, "tok3", node_id, b"data", &std::collections::HashMap::new())
            .await
            .unwrap();

        assert!(store.claim_checkpoint(&t2, "tok3").await.unwrap().is_none());
    }

    // ── Connections ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn save_and_retrieve_connection() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_connection(&t, "my-api", "My API", "rest", b"enc_data", b"nonce12345678901", "active")
            .await
            .unwrap();

        let conn = store.get_connection_by_slug(&t, "my-api").await.unwrap();
        assert!(conn.is_some());
        let (ptype, data, nonce, name, status) = conn.unwrap();
        assert_eq!(ptype, "rest");
        assert_eq!(data, b"enc_data");
        assert_eq!(nonce, b"nonce12345678901");
        assert_eq!(name, "My API");
        assert_eq!(status, "active");
    }

    #[tokio::test]
    async fn save_connection_upserts_on_conflict() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_connection(&t, "slug1", "Old", "rest", b"old_data", b"nonce12345678901", "active")
            .await
            .unwrap();
        store
            .save_connection(&t, "slug1", "New", "rest", b"new_data", b"nonce12345678901", "active")
            .await
            .unwrap();

        let conn = store.get_connection_by_slug(&t, "slug1").await.unwrap().unwrap();
        assert_eq!(conn.0, "rest");
        assert_eq!(conn.1, b"new_data");
        assert_eq!(conn.3, "New");
    }

    #[tokio::test]
    async fn tenant_isolation_connections() {
        let store = in_memory_store().await;
        let t1 = tenant();
        let t2 = other_tenant();

        store
            .save_connection(&t1, "conn", "C1", "rest", b"d1", b"n1n2n3n4n5n6n7n", "active")
            .await
            .unwrap();

        // t2 cannot read t1's connection
        assert!(store.get_connection_by_slug(&t2, "conn").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_connection() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_connection(&t, "to_del", "Del", "rest", b"d", b"n123456789012345", "active")
            .await
            .unwrap();
        store.delete_connection(&t, "to_del").await.unwrap();

        assert!(store.get_connection_by_slug(&t, "to_del").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn mark_connection_status() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_connection(&t, "sc", "SC", "rest", b"d", b"n123456789012345", "unverified")
            .await
            .unwrap();
        store.mark_connection_status(&t, "sc", "active").await.unwrap();

        let conn = store.get_connection_by_slug(&t, "sc").await.unwrap().unwrap();
        assert_eq!(conn.4, "active");
    }

    #[tokio::test]
    async fn update_connection_encrypted_data() {
        let store = in_memory_store().await;
        let t = tenant();

        store
            .save_connection(&t, "upd", "Upd", "rest", b"old", b"n123456789012345", "active")
            .await
            .unwrap();
        store
            .update_connection_encrypted_data(&t, "upd", b"new_enc", b"new_nonce_123456")
            .await
            .unwrap();

        let conn = store.get_connection_by_slug(&t, "upd").await.unwrap().unwrap();
        assert_eq!(conn.1, b"new_enc");
        assert_eq!(conn.2, b"new_nonce_123456");
        assert_eq!(conn.4, "active"); // update_connection_encrypted_data sets status = 'active'
    }
}
