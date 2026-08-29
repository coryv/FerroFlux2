## 2025-02-14 - Fix insecure file permissions for master key
**Vulnerability:** The `ferroflux.key` file (the master encryption key for AES-256-GCM) was being generated and saved with default permissions (often `0644` depending on umask) using `fs::write`.
**Learning:** Default `fs::write` does not allow specifying file permissions on Unix. This means a sensitive key file was potentially readable by other users on the system, which is a critical security vulnerability.
**Prevention:** For any sensitive file (keys, credentials), explicitly use `std::os::unix::fs::OpenOptionsExt` with `.mode(0o600)` to ensure only the owner can read/write it.
