## 2024-05-24 - Fix Weak SSRF Protection
**Vulnerability:** Weak, naive string-matching SSRF protection in `ferroflux-tools/src/primitives/request.rs` allowed bypasses via DNS rebinding and IP formatting tricks.
**Learning:** Naive URL parsing and substring matching is insufficient for SSRF protection; domain resolution is necessary to catch edge cases.
**Prevention:** Use the centralized `ferroflux_security::network::validate_url` which resolves IPs before verifying if they are blocked.
