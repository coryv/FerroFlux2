## 2025-01-24 - Missing SSRF Validation on Network Connectors
**Vulnerability:** User-provided URLs in connectors (like SSE Trigger) were not being validated against internal network ranges, allowing Server-Side Request Forgery (SSRF).
**Learning:** All modules making outbound HTTP requests (SSE, HTTP nodes, etc.) must use `ferroflux_security::network::validate_url` to properly resolve IPs and block private/loopback network access.
**Prevention:** Always use the `validate_url` helper function from the security crate before issuing any external HTTP request based on user input.
