## 2024-10-24 - [CRITICAL] Overly permissive file permissions on sensitive key files
**Vulnerability:** The master encryption key (`ferroflux.key`) was generated and written to disk using `fs::write`, which defaults to standard permissive permissions (like `0o644` on Unix).
**Learning:** Using `fs::write` is insecure for cryptographic keys or sensitive configuration files, as it allows unauthorized local users to read the keys and compromise the application's confidentiality. The same vulnerability pattern was avoided in `api_key.rs` but present in `encryption.rs`.
**Prevention:** Always use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` to explicitly set `0o600` permissions when creating files containing secrets or tokens.
