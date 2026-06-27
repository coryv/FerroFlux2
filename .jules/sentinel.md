## 2024-06-03 - SSRF Vulnerability in GraphQL Tool
**Vulnerability:** The `GraphQlTool` in `ferroflux-tools/src/primitives/graphql.rs` sent an HTTP request to user-controlled URLs without validating the target address, exposing the system to Server-Side Request Forgery (SSRF).
**Learning:** Security validation functions must be uniformly applied across all external network requests.
**Prevention:** Always invoke `crate::primitives::request::check_ssrf(url)?` before proceeding with requests in primitive tools.
