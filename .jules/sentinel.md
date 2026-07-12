## 2024-07-12 - Missing SSRF Validation in SSE Connectors
**Vulnerability:** The SSE trigger connector system (`crates/ferroflux-connectors/src/systems/sse.rs`) did not validate user-provided URLs before issuing outbound HTTP requests using `reqwest`, leading to a Critical Server-Side Request Forgery (SSRF) vulnerability.
**Learning:** Any system component that executes outbound requests based on dynamically configured URLs or user inputs must perform network validation, even if it's a long-lived streaming connection like SSE.
**Prevention:** Use `ferroflux_security::network::validate_url(&url)` consistently across all components that initialize outbound HTTP requests prior to client builder execution.
