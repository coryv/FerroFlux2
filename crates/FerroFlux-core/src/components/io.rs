use bevy_ecs::prelude::*;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for a Webhook Node (Ingest).
///
/// Acts as an entry point for external systems to trigger a workflow
/// via HTTP requests.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebhookConfig {
    /// The URL path segment to listen on.
    pub path: String,
    /// The HTTP method to accept (GET, POST, etc.).
    pub method: String,
}

/// Configuration for an HTTP Node (Connector).
///
/// Performs outbound HTTP requests to external APIs.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HttpConfig {
    /// The full target URL.
    pub url: String,
    /// The HTTP method to use.
    pub method: String,
    /// Optional key to map the response body to in the workflow state.
    #[serde(default)]
    pub result_key: Option<String>,
    /// Optional slug reference to a secure connection.
    #[serde(default)]
    pub connection_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
pub enum Frequency {
    #[default]
    Once,
    Minutes,
    Hourly,
    Daily,
    Weekly,
}

/// Configuration for a Cron Node (Time Trigger).
///
/// Initiates workflow execution based on a time schedule.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CronConfig {
    /// The recurrence interval.
    pub frequency: Frequency,
    /// The baseline time to calculate intervals from.
    #[serde(default = "default_start_at")]
    pub start_at: DateTime<Utc>,
}

fn default_start_at() -> DateTime<Utc> {
    Utc::now()
}
