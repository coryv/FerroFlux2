## 2024-09-13 - [Master Key File Permissions]
**Vulnerability:** The auto-generated master encryption key (`ferroflux.key`) was written to disk using standard `fs::write`, which applied default permissions (often `0o644`), exposing the key to unauthorized read access by other local users.
**Learning:** Enforcing restricted file permissions (0o600) on auto-generated secrets is a reusable security pattern that prevents local privilege escalation and secret leakage.
**Prevention:** Always use `std::fs::OpenOptions` with `.mode(0o600)` on Unix targets to write newly generated secret files.
