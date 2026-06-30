## 2024-06-30 - Insecure Default File Permissions for Auto-Generated Keys
**Vulnerability:** The auto-generation logic for `ferroflux.key` creates the master key file with default system permissions (often `0644`), allowing read access to other users on the system.
**Learning:** `std::fs::write` in Rust creates files with default permissions. When dealing with sensitive files like encryption keys, explicit permission models (like `0o600` on Unix) must be manually configured via `std::fs::OpenOptions` and `std::os::unix::fs::OpenOptionsExt`.
**Prevention:** Always use explicitly restricted file permissions (`0o600`) when creating files that store sensitive material such as keys or tokens.
