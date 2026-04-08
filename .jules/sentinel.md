## 2024-06-19 - File Permission Issue with Master Key Generation
**Vulnerability:** The master key generation in `ferroflux-security` was using `std::fs::write`, which created files with permissive permissions (`0o644` by default), leaving sensitive key files readable to other users on the system.
**Learning:** `std::fs::write` does not allow specifying file permissions on creation, which is a significant security risk for sensitive files like encryption keys or API tokens.
**Prevention:** Always use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` and set the mode explicitly (e.g., `mode(0o600)`) when writing sensitive files to disk on Unix systems.
