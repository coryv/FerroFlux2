## 2024-03-29 - Insecure Key Storage via Default File Permissions
**Vulnerability:** Automatically generated master encryption keys (`ferroflux.key`) and API keys (`ferroflux.api.key`) were saved to disk using `fs::write`, which applies default umask permissions (often readable by other users).
**Learning:** Security standards for this repository mandate that sensitive files be created with restricted permissions (`0o600`) to prevent unauthorized access by other local users.
**Prevention:** Use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to explicitly set `.mode(0o600)` when creating sensitive files.
