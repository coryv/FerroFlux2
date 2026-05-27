
## 2024-05-27 - [HIGH] Fix insecure file permissions for ferroflux.key
**Vulnerability:** The master encryption key `ferroflux.key` was created using `fs::write`, which uses default file permissions and could allow other users on the system to read it.
**Learning:** System primitives like `fs::write` are unsafe for handling secrets or credentials. Always enforce strict permissions when handling cryptographic materials.
**Prevention:** Use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt::mode(0o600)` to ensure secrets are created with restricted file permissions (only readable/writable by owner) on Unix platforms.
