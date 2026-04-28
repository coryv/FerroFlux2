## 2026-04-28 - Incomplete SSRF Protection
**Vulnerability:** Simple string matching for IP filtering (e.g. `host == "localhost"`) can be easily bypassed by alternative IP representations (like octal/hex) or DNS rebinding.
**Learning:** Relying on basic string parsing for security boundaries is insufficient when dealing with complex standards like URLs and IP addresses.
**Prevention:** Always use dedicated security libraries (like `ferroflux_security::network::validate_url`) that perform full DNS resolution and validation to properly enforce network boundaries.
