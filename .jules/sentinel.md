## 2024-06-01 - Fix insecure file permissions for master key
**Vulnerability:** `ferroflux.key` file was created with default OS permissions, allowing unauthorized local users to read the master encryption key.
**Learning:** `fs::write` must never be used for files containing sensitive secrets, as it defaults to standard permissions.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms (via `std::os::unix::fs::OpenOptionsExt`) to restrict file access to the owner.
