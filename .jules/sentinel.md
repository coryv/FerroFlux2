## 2026-06-19 - Missing SSRF Validation in HTTP Client Implementations
**Vulnerability:** Several places in the code that make HTTP requests (SSE stream connection, Integration action execution, OAuth2 token refresh) fail to validate URLs against SSRF (Server-Side Request Forgery). This could allow malicious actors to make requests to internal/private IPs.
**Learning:** `ferroflux_security::network::validate_url` exists but is not consistently applied across all outgoing HTTP request sites.
**Prevention:** Always use `ferroflux_security::network::validate_url` or `ferroflux_tools::primitives::request::check_ssrf` before calling `reqwest::blocking::get` or `reqwest::Client::new().get/post(...)`.
