use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

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
