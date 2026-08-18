## 2026-08-18 - Insecure File Permissions for Master Encryption Key
**Vulnerability:** The master encryption key `ferroflux.key` was created with default global read/write file permissions (e.g. 0644), potentially exposing the encryption keys to other unauthorized users on the same host.
**Learning:** Standard `std::fs::write` inherits default umask permissions which are not restrictive enough for secrets.
**Prevention:** Use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms (via `std::os::unix::fs::OpenOptionsExt`) to explicitly enforce restricted file permissions when generating files that store sensitive keys or secrets.
