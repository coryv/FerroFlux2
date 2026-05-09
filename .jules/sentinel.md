## 2024-05-09 - Insecure File Permissions on Master Key
**Vulnerability:** The master key (`ferroflux.key`) was written using `std::fs::write()`, which creates the file with default permissive permissions (`0o644` on Unix), exposing the master key to other users on the system.
**Learning:** `std::fs::write()` lacks the ability to restrict file permissions securely upon creation, leading to insecure defaults for sensitive files.
**Prevention:** For sensitive files (like keys or tokens), use `std::fs::OpenOptions` along with `std::os::unix::fs::OpenOptionsExt` to explicitly set `options.mode(0o600)` before calling `open()`.
