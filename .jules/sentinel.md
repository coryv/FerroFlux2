## 2026-05-08 - [Insecure File Permissions for Master Key]
**Vulnerability:** `ferroflux.key` generated with permissive file permissions in `crates/ferroflux-security/src/encryption.rs`, exposing the master key to unauthorized local users.
**Learning:** Default `std::fs::write` creates files with permissive modes like 0o644, which is insecure for files holding cryptographic keys.
**Prevention:** Always use `std::os::unix::fs::OpenOptionsExt` with restricted mode `0o600` when creating files containing sensitive credentials.
