## 2025-02-25 - Prevent SSRF with Network URL Validation
**Vulnerability:** The codebase relied on naive string matching against the host (e.g., `localhost`, `127.0.0.1`) for SSRF protection, which can be bypassed using obfuscated IPs or DNS rebinding.
**Learning:** Checking for SSRF using simple string matching is insufficient because attackers can bypass it using different representations of IPs or by configuring an external domain to resolve to an internal IP (DNS rebinding).
**Prevention:** Always validate URLs before issuing external HTTP requests using `ferroflux_security::network::validate_url(&url)` which properly resolves IPs and blocks internal ranges.
