## 2024-05-24 - Server-Side Request Forgery (SSRF) bypass due to weak host validation
**Vulnerability:** The custom `check_ssrf` function relied on weak string prefix checking (e.g., `host.starts_with("192.168.")` or `host == "localhost"`) rather than proper DNS resolution, leaving the system vulnerable to SSRF bypasses via DNS rebinding or IP address obfuscation.
**Learning:** String-based URL validation is never sufficient for SSRF protection because it ignores how network stacks actually resolve addresses.
**Prevention:** Always use `ferroflux_security::network::validate_url`, which properly resolves domains to IPs to block malicious network requests before they happen.
