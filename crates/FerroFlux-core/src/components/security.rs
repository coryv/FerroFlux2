use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// OAuth2 credential data stored in encrypted connection JSON.
///
/// Parsed from the `encrypted_data` blob when `auth_type == "OAuth2"`.
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

/// Configuration for Secrets Injection (Security).
///
/// Defines how to look up a sensitive value from the environment and inject it
/// into runtime requests (usually HTTP headers).
///
/// # Examples
/// * `lookup_key`: "STRIPE_KEY"
/// * `header_name`: "Authorization"
/// * `template`: "Bearer {}"
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    /// The environment variable name (e.g. "SHEETS_KEY")
    pub lookup_key: String,
    /// The target header name (e.g. "Authorization" or "X-API-KEY")
    pub header_name: String,
    /// The string template for formatting (e.g. "Bearer {}" or "{}")
    pub template: String,
}

/// Runtime Authentication Configuration for integrations.
///
/// Represents the resolved credentials or authentication method to be used
/// by a connector.
#[derive(Component, Debug, Clone)]
pub enum AuthConfig {
    /// Basic Auth (User/Pass)
    Basic {
        /// Env var for username
        user_env: String,
        /// Env var for password
        pass_env: String,
    },
    /// API Key Auth (Header or Query)
    ApiKey {
        /// Env var for the key
        key_env: String,
        /// Optional header name (if sent in header)
        header: Option<String>,
        /// Optional query param name (if sent in query string)
        query: Option<String>,
    },
    /// OAuth2 Bearer Token (Reference)
    OAuth2 {
        /// Reference to a stored token or env var
        token_ref: String,
    },
    /// Simple Bearer Token (Env Var)
    Bearer {
        /// Env var for the token
        token_env: String,
    },
}
