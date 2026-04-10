
## 2024-05-18 - [CRITICAL] Fix file permissions for auto-generated master key
**Vulnerability:** The auto-generated master key file `ferroflux.key` was created with default file permissions using `fs::write()`. This could allow other users on the system to read the master key.
**Learning:** `std::fs::write` defaults to permissive file permissions (`0o644` on Unix). Sensitive files like keys and tokens should never be created this way.
**Prevention:** Use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` to explicitly set restricted file permissions (`0o600`) when creating sensitive files on Unix platforms.
