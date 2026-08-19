## 2025-02-23 - Restrict Master Key File Permissions
**Vulnerability:** The master key file (`ferroflux.key`) was created with default system permissions, allowing local unprivileged users to read the key and decrypt stored secrets.
**Learning:** Default `std::fs::write` does not enforce secure file modes. Secrets generated to disk must explicitly be restricted.
**Prevention:** Always use `OpenOptionsExt` with `.mode(0o600)` on Unix platforms when persisting highly sensitive information like encryption keys.
