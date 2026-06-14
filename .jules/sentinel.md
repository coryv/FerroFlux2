## 2024-05-24 - Fix master key file permissions
**Vulnerability:** The master encryption key (`ferroflux.key`) was created with default file permissions via `std::fs::write`, leaving it readable by other users on the system.
**Learning:** Sensitive files must always be created with explicitly restricted permissions to prevent unauthorized access by other users on the same system.
**Prevention:** Always use `OpenOptions` with `.mode(0o600)` (via `std::os::unix::fs::OpenOptionsExt`) on Unix platforms when writing sensitive keys to disk.
