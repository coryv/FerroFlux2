## 2024-05-13 - [CRITICAL] Fix master key file permissions
**Vulnerability:** The master encryption key `ferroflux.key` was created with default file permissions (typically 0o644) instead of restricted permissions (0o600).
**Learning:** Sensitive files (keys, tokens) must be explicitly created with restricted permissions using `std::os::unix::fs::OpenOptionsExt` to prevent unauthorized read access by other users on the system.
**Prevention:** Always use `OpenOptions` with `.mode(0o600)` when creating files containing sensitive credentials or cryptographic keys on Unix platforms.
