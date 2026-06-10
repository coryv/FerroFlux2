## 2024-06-10 - Enforce restricted file permissions for sensitive keys
**Vulnerability:** Master encryption key generated via standard fs::write has overly permissive defaults, allowing unauthorized local users to read `ferroflux.key`.
**Learning:** A reusable security pattern for this project: When creating files containing sensitive secrets, API keys, or encryption keys, do not use standard std::fs::write.
**Prevention:** Instead, use std::fs::OpenOptions with .mode(0o600) on Unix platforms (via std::os::unix::fs::OpenOptionsExt) to explicitly enforce restricted file permissions.
