use ferroflux_iam::TenantId;
use anyhow::Result;
use bevy_ecs::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Row, Sqlite};
use std::str::FromStr;

#[derive(Clone, Debug, Resource)]
/// SQLite/Postgres Persistence Layer
///
/// ## Architecture: Multi-Tenancy
/// Every table (`workflows`, `checkpoints`) includes a `tenant_id` column.
/// - This enforces logical separation of data in a shared database.
/// - All queries MUST include `AND tenant_id = ?` to prevent data leaks.
pub struct PersistentStore {
    pool: Pool<Sqlite>,
}

impl PersistentStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        // Optimization for Raspberry Pi / SD Cards:
        // 1. WAL Mode: Reduces write amplification (friendly to flash storage).
        // 2. Synchronous Normal: Reduces fsync frequency while maintaining safety.
        let connection_options = SqliteConnectOptions::from_str(db_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        // Run Migration
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
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(tenant_id) REFERENCES tenants(id)
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

        // Add columns if missing (Migration for existing DB)
        // This is a naive migration check. In production use sqlx migrate!
        let _ =
            sqlx::query("ALTER TABLE workflows ADD COLUMN tenant_id TEXT DEFAULT 'default_tenant'")
                .execute(&pool)
                .await;
        let _ = sqlx::query(
            "ALTER TABLE checkpoints ADD COLUMN tenant_id TEXT DEFAULT 'default_tenant'",
        )
        .execute(&pool)
        .await;

        // Workflows Migrations
        let _ = sqlx::query("ALTER TABLE workflows ADD COLUMN status TEXT DEFAULT 'active'")
            .execute(&pool)
            .await;

        // Connections Migrations
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

        let mut workflows = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let json: String = row.get("blueprint_json");
            let status: String = row.try_get("status").unwrap_or("active".to_string());
            workflows.push((id, json, status));
        }
        Ok(workflows)
    }

    pub async fn list_workflows(
        &self,
        tenant: &TenantId,
    ) -> Result<Vec<(String, String, Option<String>, String, String)>> {
        // Returns (id, name, description, status, updated_at)
        let rows = sqlx::query(
            "SELECT id, name, description, status, updated_at FROM workflows WHERE tenant_id = ? ORDER BY updated_at DESC",
        )
        .bind(tenant.as_ref())
        .fetch_all(&self.pool)
        .await?;

        let mut workflows = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let status: String = row.try_get("status").unwrap_or("active".to_string());
            let updated_at: String = row.try_get("updated_at").unwrap_or_default();
            workflows.push((id, name, description, status, updated_at));
        }
        Ok(workflows)
    }

    pub async fn get_workflow(
        &self,
        tenant: &TenantId,
        id: &str,
    ) -> Result<Option<(String, String, Option<String>, String, String)>> {
        // Returns (id, name, description, blueprint_json, status)
        let row = sqlx::query(
            "SELECT id, name, description, blueprint_json, status FROM workflows WHERE tenant_id = ? AND id = ?",
        )
        .bind(tenant.as_ref())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let json: String = row.get("blueprint_json");
            let status: String = row.try_get("status").unwrap_or("active".to_string());
            Ok(Some((id, name, description, json, status)))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_workflow(&self, tenant: &TenantId, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM workflows WHERE tenant_id = ? AND id = ?")
            .bind(tenant.as_ref())
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

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
            r#"
            INSERT INTO checkpoints (token, node_id, data, metadata, tenant_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
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
    ) -> Result<
        Option<(
            uuid::Uuid,
            Vec<u8>,
            std::collections::HashMap<String, String>,
        )>,
    > {
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

            // Delete after read (Consume-on-read)
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

    /// Save a connection with encrypted credentials.
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

    /// Retrieve minimal info + encrypted data for a connection.
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

        if let Some(row) = row {
            let provider_type: String = row.get("provider_type");
            let data: Vec<u8> = row.get("encrypted_data");
            let nonce: Vec<u8> = row.get("nonce");
            let name: String = row.try_get("name").unwrap_or_else(|_| slug.to_string());
            let status: String = row.try_get("status").unwrap_or("unverified".to_string());
            Ok(Some((provider_type, data, nonce, name, status)))
        } else {
            Ok(None)
        }
    }

    /// Marks a connection status (e.g. "error", "active").
    pub async fn mark_connection_status(
        &self,
        tenant: &TenantId,
        slug: &str,
        status: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE connections SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE tenant_id = ? AND slug = ?")
            .bind(status)
            .bind(tenant.as_ref())
            .bind(slug)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// List connections with usage stats.
    /// Returns: Vec<(slug, name, provider_type, status, usage_count, created_at, updated_at)>
    pub async fn list_connections(
        &self,
        tenant: &TenantId,
    ) -> Result<Vec<(String, String, String, String, i64, String, String)>> {
        // Naive usage count via partial match on blueprint_json.
        // In a real app, rely on a normalized relation table or parsing the JSON proper.
        let sql = r#"
            SELECT 
                c.slug, c.name, c.provider_type, c.status, c.created_at, c.updated_at,
                (SELECT COUNT(*) FROM workflows w WHERE w.tenant_id = c.tenant_id AND w.blueprint_json LIKE '%' || c.slug || '%') as usage_count
            FROM connections c
            WHERE c.tenant_id = ?
            ORDER BY c.created_at DESC
        "#;

        let rows = sqlx::query(sql)
            .bind(tenant.as_ref())
            .fetch_all(&self.pool)
            .await?;

        let mut connections = Vec::new();
        for row in rows {
            let slug: String = row.get("slug");
            let name: String = row.try_get("name").unwrap_or_else(|_| slug.clone());
            let provider_type: String = row.get("provider_type");
            let status: String = row.try_get("status").unwrap_or("unverified".to_string());
            let created_at: String = row.try_get("created_at").unwrap_or_default();
            let updated_at: String = row.try_get("updated_at").unwrap_or_default();
            let usage_count: i64 = row.get("usage_count");

            connections.push((
                slug,
                name,
                provider_type,
                status,
                usage_count,
                created_at,
                updated_at,
            ));
        }
        Ok(connections)
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
