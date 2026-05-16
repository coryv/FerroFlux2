## 2024-05-24 - Sensitive File Creation Permissions
**Vulnerability:** The master encryption key `ferroflux.key` was created using `std::fs::write`, which defaults to permissive file modes (like `0o644` or `0o664`).
**Learning:** Standard library `fs::write` cannot guarantee secure file permissions on creation, allowing unauthorized local users potential read access to critical secrets.
**Prevention:** Always use `std::fs::OpenOptions` coupled with `std::os::unix::fs::OpenOptionsExt::mode(0o600)` when creating files containing sensitive credentials or encryption keys.
