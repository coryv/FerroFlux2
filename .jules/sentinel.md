
## 2024-05-24 - [SSRF bypasses in GraphQL and SSE]
**Vulnerability:** Found multiple occurrences in the backend where SSRF checks were missing entirely (`graphql.rs` and `sse.rs`) even when `ferroflux_security::network::validate_url` existed.
**Learning:** Even when security helpers are created, they aren't always used consistently across new modules (like custom connectors or tools).
**Prevention:** Ensure that any module issuing HTTP requests (via `reqwest` or otherwise) passes user-controlled URLs through the centralized `validate_url` check. Add tests verifying local/private IP rejection. When running local tests that *intentionally* use local endpoints, use `FERROFLUX_ALLOW_INTERNAL_IPS=1`.
