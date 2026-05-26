## 2024-05-26 - Restrict Permissions on ferroflux.key
**Vulnerability:** The master key `ferroflux.key` was created with default file permissions using `std::fs::write`, which could potentially allow other users on the system to read the master key.
**Learning:** Like `api.key`, the master key `ferroflux.key` is highly sensitive and needs explicit restricted permissions.
**Prevention:** When creating sensitive key files, always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms to enforce restricted file permissions.
