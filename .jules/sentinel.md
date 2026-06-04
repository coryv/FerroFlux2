## 2024-06-04 - Fix insecure file permissions for ferroflux.key
**Vulnerability:** The `ferroflux.key` master key file was being generated with default file permissions using `std::fs::write`, leaving the cryptographic master key potentially readable by other users on the system.
**Learning:** When automatically generating and persisting secret keys or sensitive configuration files, especially cryptograhic keys, explicit file permissions must be set. The default `fs::write` does not provide adequate protection.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix platforms (via `std::os::unix::fs::OpenOptionsExt`) to explicitly enforce restricted file permissions when creating files that store sensitive secrets.
