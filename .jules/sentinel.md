## 2024-06-05 - Restrict Permissions on Sensitive Files
**Vulnerability:** The master encryption key `ferroflux.key` was being written using `fs::write` with default file permissions, exposing it to unauthorized local users.
**Learning:** Default file creation functions like `fs::write` do not enforce secure file modes. Files containing sensitive secrets (keys, tokens) need explicit Unix permission configurations.
**Prevention:** Use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` to set `.mode(0o600)` for sensitive file creation on Unix.
