use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use rand::RngCore;
use std::env;
use std::fs;
use std::path::Path;

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
/// 2. `ferroflux.key` file in current directory (Hex encoded).
/// 3. Auto-generate new key and save to `ferroflux.key` (Dev mode).
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

    // 2. File
    let key_path = Path::new("ferroflux.key");
    if key_path.exists() {
        let content = fs::read_to_string(key_path).context("Failed to read ferroflux.key")?;
        let content = content.trim();
        let key = hex::decode(content).context("Invalid hex in ferroflux.key")?;
        if key.len() != 32 {
            return Err(anyhow::anyhow!(
                "ferroflux.key must be 32 bytes (64 hex chars)"
            ));
        }
        tracing::warn!("Using master key from local file 'ferroflux.key'");
        return Ok(key);
    }

    // 3. Auto-generate
    tracing::info!("Generating new master key -> 'ferroflux.key'");
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

        let mut file = options.open(key_path).context("Failed to open ferroflux.key file with restricted permissions")?;
        file.write_all(hex_key.as_bytes()).context("Failed to write ferroflux.key")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(key_path, hex_key).context("Failed to write ferroflux.key")?;
    }

    Ok(key.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let key = [42u8; 32];
        let data = b"Hello World";

        let (ciphertext, nonce) = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key, &nonce).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_get_or_create_master_key_permissions() {
        // Set up test file in temp directory to avoid race conditions
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join(format!("ferroflux_test_dir_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Backup original working directory
        let original_dir = std::env::current_dir().unwrap();

        // Change working directory to temp dir so get_or_create_master_key writes there
        std::env::set_current_dir(&test_dir).unwrap();

        let file_name = Path::new("ferroflux.key");
        if file_name.exists() {
            let _ = fs::remove_file(file_name);
        }

        let _key = get_or_create_master_key().expect("Failed to get or create master key");

        let metadata = fs::metadata(file_name).expect("Failed to get metadata");
        #[cfg(unix)]
        {
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            // Cleanup
            let _ = fs::remove_file(file_name);

            // Restore original working directory
            std::env::set_current_dir(original_dir).unwrap();

            assert_eq!(mode, 0o600, "File should have 0o600 permissions");
        }
        #[cfg(not(unix))]
        {
            // Cleanup
            let _ = fs::remove_file(file_name);

            // Restore original working directory
            std::env::set_current_dir(original_dir).unwrap();
        }
    }
}
