
## 2026-04-24 - SSRF in SSE Connector
**Vulnerability:** The SSE connector accepted any URL provided to it without validating it against internal IP restrictions, allowing Server-Side Request Forgery.
**Learning:** When creating long-lived connections like SSE streams, URL validation is just as necessary as it is for simple HTTP GET requests. The connection loop was entirely missing the security check.
**Prevention:** Always apply ferroflux_security::network::validate_url before spawning any reqwest clients, even for streamed or event-based connections.
