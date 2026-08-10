
## 2024-08-10 - Secure Master Key File Permissions
**Vulnerability:** The master key file `ferroflux.key` was written with default, potentially permissive file permissions using `fs::write`, risking exposure of sensitive encryption keys to unauthorized system users.
**Learning:** High-value secrets like encryption master keys must always be stored with explicitly restricted permissions (e.g., `0o600` on Unix systems) at creation time.
**Prevention:** Instead of standard `fs::write` or `File::create`, use `std::fs::OpenOptions` along with `std::os::unix::fs::OpenOptionsExt` to set `.mode(0o600)` when creating or overwriting sensitive files.
