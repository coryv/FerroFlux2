
## 2024-05-24 - Secure File Creation for Sensitive Keys
**Vulnerability:** The auto-generated master encryption key (`ferroflux.key`) was created using `fs::write`, which defaults to overly permissive file permissions (e.g. `0o644`), exposing it to other users on the system.
**Learning:** Security-sensitive files, like cryptographic keys, must be created with restricted file permissions (`0o600` on Unix) to ensure confidentiality. Using default file creation functions like `fs::write` is inadequate for such material.
**Prevention:** Always use `std::fs::OpenOptions` combined with `std::os::unix::fs::OpenOptionsExt` to explicitly set `0o600` permissions when creating or writing to sensitive files on Unix-like operating systems.
