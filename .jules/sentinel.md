
## 2024-05-24 - [Insecure Default File Permissions on Master Encryption Key]
**Vulnerability:** The auto-generated master encryption key (`ferroflux.key`) was created using `std::fs::write`, which defaults to standard permissions (typically `0o644` or similar, depending on umask). This allows any local user to read the master key.
**Learning:** Default filesystem operations in Rust (`fs::write`, `File::create`) do not enforce strict security permissions. Sensitive files like encryption keys, API tokens, and credentials must explicitly set restricted permissions during creation.
**Prevention:** Use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to explicitly set `mode(0o600)` (read/write by owner only) when creating sensitive files on Unix platforms. Avoid `std::fs::write` for any secret material.
