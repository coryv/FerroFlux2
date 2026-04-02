## 2025-02-28 - Insecure File Permissions for Sensitive Keys
**Vulnerability:** The master encryption key was being written to disk using `std::fs::write()`, which defaults to overly permissive permissions (e.g., `0o644`), allowing unauthorized read access on multi-user systems.
**Learning:** Standard file I/O operations do not guarantee secure permissions for sensitive data. Explicitly specifying restricted modes like `0o600` is necessary for secrets.
**Prevention:** Always use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt::mode(0o600)` when creating files containing sensitive materials (keys, tokens, credentials).
