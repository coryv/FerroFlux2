## 2024-05-24 - Fix Naive SSRF Protection
**Vulnerability:** Naive string matching on URLs was used to prevent SSRF, allowing bypasses via DNS resolution (e.g., resolving a public domain to a private IP).
**Learning:** Always use IP resolution to validate hostnames against private IP ranges.
**Prevention:** Use `ferroflux_security::network::validate_url(&url)` which properly resolves and validates the IP.
