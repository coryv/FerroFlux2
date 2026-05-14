
## 2024-05-18 - Fix Master Key File Permissions
**Vulnerability:** The master encryption key (`ferroflux.key`) was being created locally with default file permissions using `std::fs::write`, which exposes it to read/write by any user on the system (`0o644` depending on the umask).
**Learning:** Using generic file writing functions for sensitive key materials defaults to dangerously permissive permissions, creating local privilege escalation vulnerabilities.
**Prevention:** Always use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to explicitly set `mode(0o600)` when generating keys or secrets to ensure they are readable and writable exclusively by the owner.
