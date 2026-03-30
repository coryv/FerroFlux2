## 2024-03-30 - Insecure default file permissions when generating keys
**Vulnerability:** The master key file (`ferroflux.key`) was generated using `fs::write`, which creates files with default permissions (often `0o644`), leaving highly sensitive keys readable by other users on the system.
**Learning:** Rust's standard `fs::write` does not enforce restrictive permissions. When generating security-sensitive files such as keys or tokens on Unix systems, default permissions are insufficient and pose a local privilege escalation/data leakage risk.
**Prevention:** Always use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` to explicitly set `options.mode(0o600)` when creating sensitive files.
