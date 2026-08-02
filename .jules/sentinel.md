## 2024-05-24 - [Fix insecure file permissions for master key]
**Vulnerability:** The master key (`ferroflux.key`) was created using `fs::write` without restricted permissions, allowing any user on the system to potentially read the key.
**Learning:** Files containing sensitive secrets or encryption keys must always explicitly set restricted permissions.
**Prevention:** Use `std::fs::OpenOptions` with `.mode(0o600)` via `std::os::unix::fs::OpenOptionsExt` when writing sensitive files on Unix platforms.
