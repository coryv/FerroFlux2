## 2026-05-28 - Enforce restricted permissions for generated secrets
**Vulnerability:** Automatically generated secret files (like `ferroflux.key`) were being written with default permissions, making them readable by other users on the system.
**Learning:** This codebase frequently auto-generates keys (`api.key`, `ferroflux.key`) in the filesystem for convenience. The default `fs::write` does not enforce secure file modes.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms via `std::os::unix::fs::OpenOptionsExt` when writing sensitive secrets to disk.
