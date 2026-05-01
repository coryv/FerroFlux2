## 2024-05-01 - [Insecure Default Permissions on Master Key]
**Vulnerability:** Master key file (`ferroflux.key`) was created using `fs::write()`, which defaults to permissive file permissions (0o644) leaving the master key readable by other users.
**Learning:** `fs::write()` should never be used for sensitive files like encryption keys, API tokens, or secrets.
**Prevention:** Use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to strictly enforce `0o600` mode upon file creation for all sensitive files.
