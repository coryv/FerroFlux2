use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use rand::RngCore;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn get_master_key_path() -> PathBuf {
    // 1. Env Var override
    if let Ok(path_str) = env::var("FERROFLUX_MASTER_KEY_PATH") {
        return PathBuf::from(path_str);
    }

    // 2. Legacy fallback
    let legacy_path = PathBuf::from("ferroflux.key");
    if legacy_path.exists() {
        return legacy_path;
    }

    // 3. New default
    if let Ok(home) = env::var("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".ferroflux");
        let _ = fs::create_dir_all(&path);
        path.push("master.key");
        return path;
    }

    legacy_path
}

/// Encryption algorithm: AES-256-GCM
///
/// ## Standards
/// - **Key Size**: 32 bytes (256 bits).
/// - **Nonce Size**: 12 bytes (96 bits), randomly generated per encryption.
/// - **Tag**: 16 bytes (Auth Tag), implicitly handled by `aes-gcm` crate (appended to ciphertext).
///
/// This provides Authenticated Encryption with Associated Data (AEAD), ensuring confidentiality and integrity.
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    if key.len() != 32 {
        return Err(anyhow::anyhow!("Key must be 32 bytes"));
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(anyhow::anyhow!("Key must be 32 bytes"));
    }
    if nonce.len() != 12 {
        return Err(anyhow::anyhow!("Nonce must be 12 bytes"));
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Retrieves the master key.
///
/// Priority:
/// 1. `FERROFLUX_MASTER_KEY` environment variable (Hex encoded).
/// 2. `FERROFLUX_MASTER_KEY_PATH` override, or legacy `ferroflux.key`, or `$HOME/.ferroflux/master.key`
/// 3. Auto-generate new key and save to the determined key path (Dev mode).
#[tracing::instrument]
pub fn get_or_create_master_key() -> Result<Vec<u8>> {
    // 1. Env Var
    if let Ok(val) = env::var("FERROFLUX_MASTER_KEY") {
        let key = hex::decode(&val).context("Invalid hex in FERROFLUX_MASTER_KEY")?;
        if key.len() != 32 {
            return Err(anyhow::anyhow!(
                "FERROFLUX_MASTER_KEY must be 32 bytes (64 hex chars)"
            ));
        }
        tracing::info!("Using master key from environment variable");
        return Ok(key);
    }

    let key_path = get_master_key_path();
    get_or_create_master_key_internal(&key_path)
}

fn get_or_create_master_key_internal(key_path: &Path) -> Result<Vec<u8>> {
    // 2. File
    if key_path.exists() {
        let content = fs::read_to_string(key_path).context("Failed to read master key file")?;
        let content = content.trim();
        let key = hex::decode(content).context("Invalid hex in master key file")?;
        if key.len() != 32 {
            return Err(anyhow::anyhow!(
                "Master key must be 32 bytes (64 hex chars)"
            ));
        }
        tracing::warn!("Using master key from local file {:?}", key_path);
        return Ok(key);
    }

    // 3. Auto-generate
    tracing::info!("Generating new master key -> {:?}", key_path);
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    let hex_key = hex::encode(key);

    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        use std::io::Write;

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        options.mode(0o600);

        let mut file = options.open(key_path).context("Failed to open master key file with restricted permissions")?;
        file.write_all(hex_key.as_bytes()).context("Failed to write master key to file")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(key_path, hex_key).context("Failed to write master key file")?;
    }

    Ok(key.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_get_or_create_master_key_permissions() {
        let key_file = std::env::temp_dir().join(format!("test_master_key_{}.key", uuid::Uuid::new_v4()));

        // Ensure file doesn't exist before test
        if key_file.exists() {
            fs::remove_file(&key_file).unwrap();
        }

        let _key = get_or_create_master_key_internal(&key_file).expect("Failed to get or create master key");

        let metadata = fs::metadata(&key_file).expect("Failed to get metadata");
        #[cfg(unix)]
        {
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            // Cleanup
            fs::remove_file(&key_file).unwrap();

            assert_eq!(mode, 0o600, "File should have 0o600 permissions");
        }
    }

    #[test]
    fn test_roundtrip() {
        let key = [42u8; 32];
        let data = b"Hello World";

        let (ciphertext, nonce) = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }
}
