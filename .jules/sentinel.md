## Initial Sentinel Journal
## 2024-06-09 - Fix insecure file permissions for ferroflux.key
**Vulnerability:** The master encryption key `ferroflux.key` was created with default file permissions using `fs::write`, making it readable by other users on the system.
**Learning:** Default `fs::write` does not restrict file permissions. For sensitive keys, `std::os::unix::fs::OpenOptionsExt` must be used explicitly.
**Prevention:** Always use `OpenOptions` with `.mode(0o600)` when creating files that contain secrets like encryption keys or API tokens.
