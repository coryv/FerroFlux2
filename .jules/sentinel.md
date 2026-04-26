
## 2024-04-26 - Insecure file permissions with std::fs::write
**Vulnerability:** The master key file (`ferroflux.key`) was being created using `std::fs::write` which defaults to permissive `0o644` permissions, making it readable by other users on the system.
**Learning:** `std::fs::write` is unsafe for sensitive files like encryption keys or tokens.
**Prevention:** Always use `std::fs::OpenOptions` with `std::os::unix::fs::OpenOptionsExt` and set `options.mode(0o600)` when creating files containing secrets to restrict access to the owner.
