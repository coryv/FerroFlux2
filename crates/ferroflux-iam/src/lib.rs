use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Row, Sqlite};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TenantId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TenantId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for TenantId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Clone, Debug)]
pub struct IamStore {
    pool: Pool<Sqlite>,
}

impl IamStore {
    pub async fn new(db_url: &str) -> Result<Self> {
        let connection_options = SqliteConnectOptions::from_str(db_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connection_options)
            .await?;

        // Run Migration for IAM Tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS tenants (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL, -- 'personal' or 'organization'
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS user_tenants (
                user_id TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'viewer', -- 'owner', 'admin', 'editor', 'viewer'
                PRIMARY KEY (user_id, tenant_id),
                FOREIGN KEY(user_id) REFERENCES users(id),
                FOREIGN KEY(tenant_id) REFERENCES tenants(id)
            );
            CREATE TABLE IF NOT EXISTS magic_links (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                email TEXT NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn create_magic_link(&self, email: &str) -> Result<(String, String)> {
        let user_id = self.get_or_create_user_by_email(email).await?;
        let token = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

        sqlx::query(
            "INSERT INTO magic_links (token, user_id, email, expires_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&token)
        .bind(&user_id)
        .bind(email)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok((token, user_id))
    }

    pub async fn verify_magic_link_token(&self, token: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT user_id, expires_at FROM magic_links WHERE token = ?")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let user_id: String = row.get("user_id");
            let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");

            if chrono::Utc::now() > expires_at {
                self.delete_magic_link(token).await?;
                return Ok(None);
            }

            self.delete_magic_link(token).await?;
            self.ensure_personal_tenant(&user_id).await?;

            Ok(Some(user_id))
        } else {
            Ok(None)
        }
    }

    async fn delete_magic_link(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM magic_links WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_or_create_user_by_email(&self, email: &str) -> Result<String> {
        let row = sqlx::query("SELECT id FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(row.get("id"))
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO users (id, email) VALUES (?, ?)")
                .bind(&id)
                .bind(email)
                .execute(&self.pool)
                .await?;
            Ok(id)
        }
    }

    async fn ensure_personal_tenant(&self, user_id: &str) -> Result<()> {
        let has_personal = sqlx::query(
            r#"
            SELECT 1 FROM tenants t
            JOIN user_tenants ut ON t.id = ut.tenant_id
            WHERE ut.user_id = ? AND t.type = 'personal'
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .is_some();

        if !has_personal {
            let tenant_id = Uuid::new_v4().to_string();
            let name = "Personal Workspace";

            let mut tx = self.pool.begin().await?;

            sqlx::query("INSERT INTO tenants (id, name, type) VALUES (?, ?, 'personal')")
                .bind(&tenant_id)
                .bind(name)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                "INSERT INTO user_tenants (user_id, tenant_id, role) VALUES (?, ?, 'owner')",
            )
            .bind(user_id)
            .bind(&tenant_id)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        }
        Ok(())
    }

    pub async fn get_user_tenants(&self, user_id: &str) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.name, t.type 
            FROM tenants t
            JOIN user_tenants ut ON t.id = ut.tenant_id
            WHERE ut.user_id = ?
            ORDER BY t.created_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut res = Vec::new();
        for row in rows {
            res.push((row.get("id"), row.get("name"), row.get("type")));
        }
        Ok(res)
    }

    pub async fn get_user_email(&self, user_id: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT email FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.get("email")))
    }

    pub async fn is_user_in_tenant(&self, user_id: &str, tenant_id: &str) -> Result<bool> {
        let row = sqlx::query("SELECT 1 FROM user_tenants WHERE user_id = ? AND tenant_id = ?")
            .bind(user_id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }
}
