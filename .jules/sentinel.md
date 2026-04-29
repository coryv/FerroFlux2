## 2024-04-29 - [Restricted Permissions for Master Key]
**Vulnerability:** The AES-256-GCM master key (`ferroflux.key`) was being created using `fs::write()`, which defaults to overly permissive file permissions on Unix systems.
**Learning:** `fs::write()` is unsafe for creating sensitive files. Even auto-generated keys must use explicit permission boundaries.
**Prevention:** Always use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to enforce `0o600` permissions when creating files that contain sensitive data or credentials.
