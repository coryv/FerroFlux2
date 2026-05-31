## 2024-05-31 - Fix vulnerable file permissions for master key
**Vulnerability:** The auto-generated master key (`ferroflux.key`) was created with default file permissions using `fs::write()`, exposing the key to other users on the system.
**Learning:** System APIs like `std::fs::write` use default umask permissions, which is unsafe for sensitive secrets.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` via `std::os::unix::fs::OpenOptionsExt` when auto-generating cryptographic keys or secrets.
