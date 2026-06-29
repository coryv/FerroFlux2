## 2024-05-30 - Fix SSRF Vulnerability in GraphQL Tool
**Vulnerability:** The `graphql` primitive tool accepted user-provided URLs and executed HTTP requests against them without validating if the URL pointed to an internal IP address (like 127.0.0.1 or 10.0.0.0/8), allowing Server-Side Request Forgery (SSRF).
**Learning:** Tools that execute arbitrary network requests must enforce the `FERROFLUX_ALLOW_INTERNAL_IPS` environment variable check and IP blocklist to prevent SSRF in the FerroFlux execution environment.
**Prevention:** Use `ferroflux_security::network::validate_url` or `crate::primitives::request::check_ssrf(url)?` consistently across all new connectors, tools, or triggers before constructing `reqwest` clients.
