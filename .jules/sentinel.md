## 2024-07-07 - Insecure Permissions on Master Key Generation
**Vulnerability:** The master encryption key `ferroflux.key` was generated and written to disk using standard `fs::write` without restricting file permissions, leaving it readable by other users on the system.
**Learning:** System-generated sensitive files (like encryption or API keys) must explicitly restrict permissions on creation to prevent unauthorized read access by local users.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms via `std::os::unix::fs::OpenOptionsExt` when writing files that contain secrets.
