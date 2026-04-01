## 2024-05-20 - [Fix Insecure File Permissions in `encryption.rs`]
**Vulnerability:** The master key file (`ferroflux.key`) was generated with permissive file permissions (e.g. `0o644`) on Unix systems because it used `std::fs::write`, which could allow unauthorized local users to read the key.
**Learning:** `std::fs::write` is not suitable for secrets as it doesn't guarantee restricted file permissions.
**Prevention:** Always explicitly set file permissions using `std::fs::OpenOptions` and `std::os::unix::fs::OpenOptionsExt` (e.g. `options.mode(0o600)`) when writing sensitive data to disk, similar to the existing implementation in `api_key.rs`.
