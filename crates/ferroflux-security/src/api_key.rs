use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn get_api_key_path() -> PathBuf {
    if let Ok(path_str) = env::var("FERROFLUX_API_KEY_PATH") {
        return PathBuf::from(path_str);
    }

    if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".ferroflux");
        let _ = fs::create_dir_all(&path);
        path.push("api.key");
        return path;
    }

    PathBuf::from("ferroflux.api.key")
}

/// Retrieves the API key.
///
/// Priority:
/// 1. `FERROFLUX_API_KEY` environment variable.
/// 2. Key file from `FERROFLUX_API_KEY_PATH` or default `$HOME/.ferroflux/api.key`.
/// 3. Auto-generate new UUID and save to the key file with restricted permissions.
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
    let key_path = get_api_key_path();
    if key_path.exists() {
        let content = fs::read_to_string(&key_path).context("Failed to read API key file")?;
        let key = content.trim().to_string();
        if !key.is_empty() {
            tracing::info!("Using API key from local file {:?}", key_path);
            return Ok(key);
        }
    }

    // 3. Auto-generate
    tracing::info!("Generating new API key -> {:?}", key_path);
    let key = uuid::Uuid::new_v4().to_string();

    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        use std::io::Write;

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        options.mode(0o600);

        let mut file = options.open(&key_path).context("Failed to open API key file with restricted permissions")?;
        file.write_all(key.as_bytes()).context("Failed to write API key to file")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&key_path, &key).context("Failed to write API key to file")?;
    }

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_get_or_create_api_key_permissions() {
        let key_file = PathBuf::from("test_api.key");
        env::set_var("FERROFLUX_API_KEY_PATH", &key_file);

        if key_file.exists() {
            fs::remove_file(&key_file).unwrap();
        }

        let _key = get_or_create_api_key().expect("Failed to get or create API key");

        let metadata = fs::metadata(&key_file).expect("Failed to get metadata");
        #[cfg(unix)]
        {
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            // Cleanup
            fs::remove_file(&key_file).unwrap();
            env::remove_var("FERROFLUX_API_KEY_PATH");

            assert_eq!(mode, 0o600, "File should have 0o600 permissions");
        }
    }
}
