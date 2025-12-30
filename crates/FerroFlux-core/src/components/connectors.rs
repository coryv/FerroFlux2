use bevy_ecs::prelude::Component;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Configuration for an RSS Feed Ingestion Node.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RssConfig {
    /// The URL of the RSS feed.
    pub url: String,
    /// Polling interval in seconds.
    pub interval_seconds: u64,
}

#[derive(Component, Debug, Clone, Default)]
pub struct RssState {
    pub last_pub_date: Option<SystemTime>,
}

/// Configuration for an XML parsing/transformation node.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct XmlConfig {
    /// Field containing the XML string. If None, assumes the entire payload is the XML string.
    pub target_field: Option<String>,
    /// Field to write the JSON result to.
    pub result_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum FtpProtocol {
    Ftp,
    Sftp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum FtpOperation {
    List,
    Get,
    Put,
}

/// Configuration for an FTP/SFTP Connector Node.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FtpConfig {
    /// The protocol to use (FTP or SFTP).
    pub protocol: FtpProtocol,
    /// The remote host address.
    pub host: String,
    /// The remote port.
    pub port: u16,
    /// Env var for username.
    pub user_secret: String,
    /// Env var for password.
    pub pass_secret: String,
    /// The operation to perform (List, Get, Put).
    pub operation: FtpOperation,
    /// The remote path to operate on.
    pub path: String,
    /// Optional slug reference to a secure connection.
    #[serde(default)]
    pub connection_slug: Option<String>,
}

/// Configuration for an SSH Command Execution Node.
#[derive(Component, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SshConfig {
    /// The remote host address.
    pub host: String,
    /// The remote port (usually 22).
    pub port: u16,
    /// Env var for username.
    pub user_secret: String,
    /// Env var for password or private key path.
    /// Can be a password or a path to a private key, depending on usage context.
    pub key_secret: String,
    /// The command to execute on the remote shell.
    pub command: String,
    /// Optional slug reference to a secure connection.
    #[serde(default)]
    pub connection_slug: Option<String>,
}
