## 2024-05-18 - [HIGH] Insecure file permissions for auto-generated master key
**Vulnerability:** The `get_or_create_master_key` function generates a new master key (`ferroflux.key`) in dev mode, saving it using standard `fs::write()`. This relies on the system's default umask and potentially creates the file with broad permissions (e.g., `0o644` readable by other users).
**Learning:** Even for local dev features or fallbacks, files containing highly sensitive secrets must be created with explicitly restricted permissions to prevent local privilege escalation or secret leakage.
**Prevention:** Always use `std::os::unix::fs::OpenOptionsExt` with `.mode(0o600)` when creating files that store secrets, rather than relying on default `std::fs::write`.
