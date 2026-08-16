## 2024-10-25 - Insecure file permissions for master encryption key
**Vulnerability:** The auto-generated master encryption key (`ferroflux.key`) was created using `std::fs::write`, which applies default umask permissions, making the sensitive key readable by other users on the system.
**Learning:** Keys and secrets written to the filesystem must explicitly have their permissions restricted to `0o600` on Unix systems to ensure only the owner can read/write them.
**Prevention:** Always use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` `.mode(0o600)` when writing secret files on Unix platforms, instead of `std::fs::write`.
