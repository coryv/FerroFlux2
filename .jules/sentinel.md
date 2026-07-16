## 2024-07-16 - [Restrict file permissions for secret files]
**Vulnerability:** The auto-generated ferroflux.key was written using std::fs::write without explicit file permissions, meaning it could be readable by other system users.
**Learning:** When creating files containing sensitive secrets, API keys, or encryption keys (like ferroflux.key), standard std::fs::write is insufficient as it applies default permissions.
**Prevention:** Always use std::fs::OpenOptions with .mode(0o600) on Unix platforms (via std::os::unix::fs::OpenOptionsExt) to explicitly enforce restricted file permissions.
