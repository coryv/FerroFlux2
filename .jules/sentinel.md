## 2024-05-24 - File Permission Issue in Master Key Generation
**Vulnerability:** The auto-generated `ferroflux.key` was created using standard `std::fs::write`, which relies on the system umask and may result in world-readable permissions (e.g., `0o644`), exposing the encryption master key.
**Learning:** Keys generated dynamically on disk need explicit permissions. `fs::write` is insecure for secrets.
**Prevention:** Use `std::fs::OpenOptions` with `mode(0o600)` (via `std::os::unix::fs::OpenOptionsExt`) when creating files containing sensitive secrets or keys.
