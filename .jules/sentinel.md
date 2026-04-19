## 2025-02-12 - Insecure master key file permissions
**Vulnerability:** Master key file `ferroflux.key` was created with default permissive permissions (`0o644` or `0o666`), potentially allowing unauthorized access to the encrypted database secrets.
**Learning:** `std::fs::write` creates files with overly permissive permissions. Sensitive files like keys and tokens must have restricted permissions.
**Prevention:** Use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` to explicitly set `0o600` permissions when creating sensitive files.
