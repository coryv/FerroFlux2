## 2024-05-24 - Insecure Master Key File Permissions
**Vulnerability:** The 'ferroflux.key' master key file was created with default file permissions, making it readable by other users on the system.
**Learning:** Cryptographic material generated at runtime must explicitly have restricted permissions (0o600) set at the moment of creation using `OpenOptionsExt`.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms instead of `std::fs::write` when creating files containing sensitive secrets, API keys, or encryption keys.
