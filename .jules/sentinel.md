## 2024-05-04 - [Insecure Permissions on Auto-Generated Master Key]
**Vulnerability:** The `ferroflux.key` master key was auto-generated using `std::fs::write`, which creates the file with default permissive permissions (typically 0o644 or 0o666), exposing the master key to other local users.
**Learning:** When generating sensitive files like cryptographic keys or API tokens, `std::fs::write` must be avoided as it does not allow setting strict permissions.
**Prevention:** Always use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` (on Unix) to explicitly enforce `0o600` permissions when creating files containing sensitive secrets.
