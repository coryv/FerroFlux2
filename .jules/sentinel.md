## 2025-02-23 - Insecure File Permissions on Master Key Generation
**Vulnerability:** The master key for encryption (`ferroflux.key`) was created using `std::fs::write`, which uses default permissive file permissions (e.g., `0o644` depending on umask) on Unix systems, allowing unauthorized users on the same machine to potentially read the encryption key.
**Learning:** In Rust, sensitive files like encryption keys, tokens, or credentials must be explicitly created with restricted permissions. Relying on default filesystem behaviors exposes critical secrets locally.
**Prevention:** Always use `std::fs::OpenOptions` along with `std::os::unix::fs::OpenOptionsExt` to explicitly set file modes to `0o600` when creating files containing sensitive information.
