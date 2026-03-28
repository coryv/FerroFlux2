//! OAuth2 Token Refresh Support
//!
//! Provides automatic access token refresh for OAuth2 connections.

use crate::secrets::{DatabaseSecretStore, SecretStore};
use anyhow::{anyhow, Context, Result};
use bevy_ecs::prelude::*;
use dashmap::DashMap;
use ferroflux_types::resources::TokioRuntime;
use ferroflux_types::tenant::TenantId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// OAuth2 credential data stored in encrypted connection JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Credentials {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix epoch seconds when the access token expires.
    pub expires_at: i64,
    /// The token endpoint for refreshing (e.g., "https://oauth2.googleapis.com/token").
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(default)]
    pub scopes: Option<String>,
}

/// Response from an OAuth2 token refresh request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    /// Seconds until the new token expires.
    pub expires_in: Option<u64>,
    /// Some providers rotate refresh tokens on each use.
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
}

const EXPIRY_BUFFER_SECS: i64 = 60;

#[derive(Resource, Default, Clone)]
pub struct TokenRefreshLocks {
    locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl TokenRefreshLocks {
    pub fn get_lock(&self, tenant: &TenantId, slug: &str) -> Arc<Mutex<()>> {
        let key = format!("{}:{}", tenant.as_ref(), slug);
        self.locks
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

pub fn is_token_expired(creds: &OAuth2Credentials) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    creds.expires_at <= now + EXPIRY_BUFFER_SECS
}

async fn refresh_access_token(creds: &OAuth2Credentials) -> Result<OAuth2TokenResponse> {
    let client = reqwest::Client::new();

    let mut form = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", &creds.refresh_token),
        ("client_id", &creds.client_id),
        ("client_secret", &creds.client_secret),
    ];

    if let Some(ref scopes) = creds.scopes {
        form.push(("scope", scopes));
    }

    let resp = client
        .post(&creds.token_url)
        .form(&form)
        .send()
        .await
        .context("OAuth2 refresh request failed")?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "OAuth2 token refresh failed (HTTP {}): {}",
            status.as_u16(),
            body
        ));
    }

    resp.json::<OAuth2TokenResponse>()
        .await
        .context("Failed to parse OAuth2 token response")
}

fn merge_token_into_conn_data(conn_data: &Value, token: &OAuth2TokenResponse) -> Result<Value> {
    let mut updated = conn_data.clone();
    let obj = updated
        .as_object_mut()
        .ok_or_else(|| anyhow!("Connection data is not a JSON object"))?;

    obj.insert(
        "access_token".to_string(),
        Value::String(token.access_token.clone()),
    );

    let expires_in = token.expires_in.unwrap_or(3600);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    obj.insert(
        "expires_at".to_string(),
        Value::Number((now + expires_in).into()),
    );

    if let Some(ref new_refresh) = token.refresh_token {
        obj.insert(
            "refresh_token".to_string(),
            Value::String(new_refresh.clone()),
        );
    }

    Ok(updated)
}

pub fn resolve_oauth2_token(
    tenant: &TenantId,
    slug: &str,
    conn_data: &Value,
    store: &DatabaseSecretStore,
    rt: &TokioRuntime,
    locks: Option<&TokenRefreshLocks>,
) -> Result<String> {
    let creds: OAuth2Credentials = serde_json::from_value(conn_data.clone())
        .context("Failed to parse OAuth2 credentials from connection data")?;

    if !is_token_expired(&creds) {
        return Ok(creds.access_token);
    }

    tracing::info!(
        tenant = tenant.as_ref(),
        connection = slug,
        "OAuth2 access token expired, attempting refresh"
    );

    let lock = locks
        .map(|l| l.get_lock(tenant, slug))
        .unwrap_or_else(|| Arc::new(Mutex::new(())));

    rt.0.block_on(async {
        let _guard = lock.lock().await;

        let fresh_conn_data = store.resolve_connection(tenant, slug).await?;
        let fresh_creds: OAuth2Credentials = serde_json::from_value(fresh_conn_data.clone())
            .context("Failed to parse refreshed OAuth2 credentials")?;

        if !is_token_expired(&fresh_creds) {
            tracing::debug!(
                connection = slug,
                "OAuth2 token was refreshed by another thread"
            );
            return Ok(fresh_creds.access_token);
        }

        let token_response = refresh_access_token(&fresh_creds).await?;
        let updated_conn = merge_token_into_conn_data(&fresh_conn_data, &token_response)?;

        let updated_bytes = serde_json::to_vec(&updated_conn)?;
        store
            .update_connection_data(tenant, slug, &updated_bytes)
            .await
            .context("Failed to persist refreshed OAuth2 tokens")?;

        tracing::info!(
            connection = slug,
            "OAuth2 token refreshed and persisted successfully"
        );

        Ok(token_response.access_token)
    })
}
