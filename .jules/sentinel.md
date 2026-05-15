## Sentinel Journal
## 2026-05-15 - Insecure file permissions on sensitive key files
**Vulnerability:** The auto-generated master key (`ferroflux.key`) was created using `std::fs::write`, which defaults to permissive file permissions (e.g., 0o644) on UNIX systems, exposing the secret to other users on the system.
**Learning:** `std::fs::write` is unsafe for sensitive files on UNIX systems. It should be avoided when storing API keys, passwords, or encryption keys.
**Prevention:** Always use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` to explicitly set secure file permissions (e.g., `0o600`) when creating sensitive files.
