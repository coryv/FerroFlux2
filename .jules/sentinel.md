## 2024-05-15 - Insecure File Permissions for Master Key
**Vulnerability:** The auto-generated master key (`ferroflux.key`) was created using `std::fs::write`, which leaves the file with default system permissions (often readable by other users).
**Learning:** Creating sensitive secret files (like keys or API tokens) using `std::fs::write` is insecure by default.
**Prevention:** Use `std::fs::OpenOptions` with `.mode(0o600)` (via `std::os::unix::fs::OpenOptionsExt`) to explicitly enforce restricted permissions when generating files that contain sensitive secrets.
