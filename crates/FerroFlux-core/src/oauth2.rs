//! OAuth2 Token Refresh Support
//!
//! Provides automatic access token refresh for OAuth2 connections.
//! When an access token is expired, the refresh token is used to obtain
//! a new one before executing the HTTP request.

use crate::components::security::{OAuth2Credentials, OAuth2TokenResponse};
use crate::resources::TokioRuntime;
use crate::secrets::{DatabaseSecretStore, SecretStore};
use anyhow::{Context, Result, anyhow};
use bevy_ecs::prelude::*;
use dashmap::DashMap;
use ferroflux_iam::TenantId;
use serde_json::Value;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Safety buffer (seconds) before the actual expiry time.
/// We refresh 60 seconds early to avoid races where the token expires mid-request.
const EXPIRY_BUFFER_SECS: i64 = 60;

/// Per-connection mutex to prevent concurrent refresh attempts on the same token.
///
/// Keyed by `"{tenant_id}:{slug}"`. Ensures only one thread performs a refresh
/// at a time — important because some providers invalidate the old refresh token
/// on each use.
#[derive(Resource, Default, Clone)]
pub struct TokenRefreshLocks {
    locks: Arc<DashMap<String, Arc<Mutex<()>>>>,
}

impl TokenRefreshLocks {
    fn get_lock(&self, tenant: &TenantId, slug: &str) -> Arc<Mutex<()>> {
        let key = format!("{}:{}", tenant.as_ref(), slug);
        self.locks
            .entry(key)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

/// Returns `true` if the access token is expired (or will expire within the buffer window).
pub fn is_token_expired(creds: &OAuth2Credentials) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    creds.expires_at <= now + EXPIRY_BUFFER_SECS
}

/// Performs an OAuth2 token refresh by POSTing to the token endpoint.
async fn refresh_access_token(creds: &OAuth2Credentials) -> Result<OAuth2TokenResponse> {
    let client = reqwest::Client::new();

    let mut form = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", &creds.refresh_token),
        ("client_id", &creds.client_id),
        ("client_secret", &creds.client_secret),
    ];

    // Some providers require scopes on refresh
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

/// Merges a fresh token response into the existing connection JSON.
///
/// Updates `access_token`, `expires_at`, and optionally `refresh_token`
/// (if the provider rotated it).
fn merge_token_into_conn_data(conn_data: &Value, token: &OAuth2TokenResponse) -> Result<Value> {
    let mut updated = conn_data.clone();
    let obj = updated
        .as_object_mut()
        .ok_or_else(|| anyhow!("Connection data is not a JSON object"))?;

    obj.insert(
        "access_token".to_string(),
        Value::String(token.access_token.clone()),
    );

    // Calculate new expires_at from expires_in (default 3600s if not provided)
    let expires_in = token.expires_in.unwrap_or(3600);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    obj.insert(
        "expires_at".to_string(),
        Value::Number((now + expires_in).into()),
    );

    // Update refresh_token if the provider rotated it
    if let Some(ref new_refresh) = token.refresh_token {
        obj.insert(
            "refresh_token".to_string(),
            Value::String(new_refresh.clone()),
        );
    }

    Ok(updated)
}

/// Resolve an OAuth2 connection's access token, refreshing if expired.
///
/// This is the main entry point called from `http_client.rs`. It is a **sync**
/// function that bridges to async via the Tokio runtime handle.
///
/// Flow:
/// 1. Parse `OAuth2Credentials` from the connection data
/// 2. If the token is still valid, return it immediately
/// 3. If expired, acquire a per-connection lock to prevent concurrent refreshes
/// 4. Re-read the connection (another thread may have refreshed while we waited)
/// 5. If still expired, perform the refresh, persist the new tokens, return
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

    // Fast path: token is still valid
    if !is_token_expired(&creds) {
        return Ok(creds.access_token);
    }

    tracing::info!(
        tenant = tenant.as_ref(),
        connection = slug,
        "OAuth2 access token expired, attempting refresh"
    );

    // Acquire per-connection lock to prevent concurrent refreshes
    let lock = locks
        .map(|l| l.get_lock(tenant, slug))
        .unwrap_or_else(|| Arc::new(Mutex::new(())));

    rt.0.block_on(async {
        let _guard = lock.lock().await;

        // Re-read connection after acquiring lock — another thread may have refreshed
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

        // Actually perform the refresh
        let token_response = refresh_access_token(&fresh_creds).await?;
        let updated_conn = merge_token_into_conn_data(&fresh_conn_data, &token_response)?;

        // Persist the refreshed tokens
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_not_expired() {
        let future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600; // 1 hour from now

        let creds = OAuth2Credentials {
            access_token: "test".into(),
            refresh_token: "refresh".into(),
            expires_at: future,
            token_url: "https://example.com/token".into(),
            client_id: "cid".into(),
            client_secret: "csecret".into(),
            scopes: None,
        };

        assert!(!is_token_expired(&creds));
    }

    #[test]
    fn test_token_expired() {
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - 100; // 100 seconds ago

        let creds = OAuth2Credentials {
            access_token: "test".into(),
            refresh_token: "refresh".into(),
            expires_at: past,
            token_url: "https://example.com/token".into(),
            client_id: "cid".into(),
            client_secret: "csecret".into(),
            scopes: None,
        };

        assert!(is_token_expired(&creds));
    }

    #[test]
    fn test_token_within_buffer_window() {
        // Token expires in 30 seconds — within the 60-second buffer, so should be "expired"
        let near_future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 30;

        let creds = OAuth2Credentials {
            access_token: "test".into(),
            refresh_token: "refresh".into(),
            expires_at: near_future,
            token_url: "https://example.com/token".into(),
            client_id: "cid".into(),
            client_secret: "csecret".into(),
            scopes: None,
        };

        assert!(is_token_expired(&creds));
    }

    #[test]
    fn test_merge_token_into_conn_data() {
        let conn_data = serde_json::json!({
            "auth_type": "OAuth2",
            "access_token": "old_token",
            "refresh_token": "old_refresh",
            "expires_at": 1000,
            "token_url": "https://example.com/token",
            "client_id": "cid",
            "client_secret": "csecret",
            "base_url": "https://api.example.com",
            "custom_headers": {}
        });

        let token_response = OAuth2TokenResponse {
            access_token: "new_token".into(),
            expires_in: Some(3600),
            refresh_token: Some("new_refresh".into()),
            token_type: Some("Bearer".into()),
            scope: None,
        };

        let merged = merge_token_into_conn_data(&conn_data, &token_response).unwrap();
        assert_eq!(merged["access_token"], "new_token");
        assert_eq!(merged["refresh_token"], "new_refresh");
        assert!(merged["expires_at"].as_u64().unwrap() > 1000);
        // Ensure other fields are preserved
        assert_eq!(merged["base_url"], "https://api.example.com");
        assert_eq!(merged["auth_type"], "OAuth2");
    }

    #[test]
    fn test_merge_token_without_refresh_rotation() {
        let conn_data = serde_json::json!({
            "auth_type": "OAuth2",
            "access_token": "old_token",
            "refresh_token": "keep_this",
            "expires_at": 1000,
            "token_url": "https://example.com/token",
            "client_id": "cid",
            "client_secret": "csecret"
        });

        let token_response = OAuth2TokenResponse {
            access_token: "new_token".into(),
            expires_in: Some(7200),
            refresh_token: None, // Provider didn't rotate
            token_type: None,
            scope: None,
        };

        let merged = merge_token_into_conn_data(&conn_data, &token_response).unwrap();
        assert_eq!(merged["access_token"], "new_token");
        assert_eq!(merged["refresh_token"], "keep_this"); // Preserved
    }
}
