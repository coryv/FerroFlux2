## 2026-05-25 - Fix Insecure Master Key File Permissions
**Vulnerability:** The auto-generated `ferroflux.key` master key file was created using standard `fs::write`, which defaults to overly permissive permissions (e.g., 0o644) on Unix systems, potentially allowing unauthorized local users to read the master encryption key.
**Learning:** Standard file system APIs do not restrict file access by default. Sensitive files like encryption keys, API keys, or secrets must explicitly use restricted permissions when created.
**Prevention:** Always use `std::os::unix::fs::OpenOptionsExt` with `.mode(0o600)` when writing secret files on Unix-like platforms.
