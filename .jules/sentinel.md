## 2024-06-07 - Insecure File Permissions for Encryption Keys
**Vulnerability:** `ferroflux.key` was created with default file permissions using `std::fs::write`, which could allow unauthorized local users to read the master encryption key.
**Learning:** Even internal encryption keys must be created with explicit restricted permissions (`0o600`) to prevent privilege escalation or data leakage on multi-user systems.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` via `std::os::unix::fs::OpenOptionsExt` when creating files that contain sensitive secrets, API keys, or encryption keys.
