## 2024-05-24 - [Fix SSRF vulnerability in request primitives]
**Vulnerability:** The SSRF check in `ferroflux-tools` used naive string matching (`host == "127.0.0.1" || ...`) which failed to block IPv6 localhost representations (`[::1]`, `[::ffff:127.0.0.1]`), octal/hex encodings, or external hostnames resolving to local IPs.
**Learning:** Naive string matching for URLs is inadequate to prevent SSRF due to the multiple ways IP addresses can be encoded and resolved.
**Prevention:** Always use `ferroflux_security::network::validate_url` to perform comprehensive IP resolution and network range validation before issuing external requests.
