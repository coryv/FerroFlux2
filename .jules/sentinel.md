## 2024-05-24 - Insecure Master Key File Permissions
**Vulnerability:** The encryption master key file (`ferroflux.key`) was created using `fs::write` which applies default (potentially world-readable) file permissions.
**Learning:** When generating files that store highly sensitive cryptographic secrets on disk, standard file operations are insufficient. This was a project-specific pattern where `api.key` was secured but `ferroflux.key` was missed.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` via `std::os::unix::fs::OpenOptionsExt` for any key or secret file generation.
