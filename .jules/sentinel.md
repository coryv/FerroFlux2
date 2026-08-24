## 2024-10-25 - Fix SSRF in SSE connectors
**Vulnerability:** The SSE trigger system in `ferroflux-connectors` did not validate URLs, allowing Server-Side Request Forgery (SSRF).
**Learning:** Connectors that fetch URLs on behalf of users must always validate the URL against private IP ranges.
**Prevention:** Use `ferroflux_security::network::validate_url` before making any external HTTP requests.
