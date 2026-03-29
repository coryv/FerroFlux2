use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::io::Write;

/// Retrieves the API key.
///
/// Priority:
/// 1. `FERROFLUX_API_KEY` environment variable.
/// 2. `ferroflux.api.key` file in current directory.
/// 3. Auto-generate new UUID and save to `ferroflux.api.key`.
#[tracing::instrument]
pub fn get_or_create_api_key() -> Result<String> {
    // 1. Env Var
    if let Ok(val) = env::var("FERROFLUX_API_KEY") {
        if !val.is_empty() {
            tracing::info!("Using API key from environment variable");
            return Ok(val);
        }
    }

    // 2. File
    let key_path = Path::new("ferroflux.api.key");
    if key_path.exists() {
        let content = fs::read_to_string(key_path).context("Failed to read ferroflux.api.key")?;
        let key = content.trim().to_string();
        if !key.is_empty() {
            tracing::info!("Using API key from local file 'ferroflux.api.key'");
            return Ok(key);
        }
    }

    // 3. Auto-generate
    tracing::info!("Generating new API key -> 'ferroflux.api.key'");
    let key = uuid::Uuid::new_v4().to_string();

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(key_path)
        .context("Failed to open ferroflux.api.key with secure permissions")?;

    file.write_all(key.as_bytes())
        .context("Failed to write ferroflux.api.key")?;

    Ok(key)
}
