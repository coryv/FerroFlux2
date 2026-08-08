## 2024-08-08 - Insecure Master Key File Permissions
**Vulnerability:** The auto-generated master key (`ferroflux.key`) was created using `std::fs::write`, which applies default umask permissions, leaving the encryption key potentially readable by other users on the system.
**Learning:** Sensitive keys created locally should explicitly define restricted file permissions (e.g., `0o600`) using `std::fs::OpenOptions` and `std::os::unix::fs::OpenOptionsExt`.
**Prevention:** Avoid `std::fs::write` for files containing secrets. Use platform-specific APIs to lock down read/write access.
