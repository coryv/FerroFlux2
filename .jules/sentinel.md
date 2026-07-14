## 2026-07-14 - Insecure file permissions for master key
**Vulnerability:** The master key file `ferroflux.key` was created with default file permissions using `fs::write`, potentially exposing the key to other users on the system.
**Learning:** Critical secrets, such as API keys and encryption keys, should not be written to disk using standard `fs::write`. They require explicit OS-level restricted permissions (e.g., 0o600 on Unix) to prevent unauthorized access.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` (via `std::os::unix::fs::OpenOptionsExt`) when creating files containing sensitive secrets on Unix platforms.
