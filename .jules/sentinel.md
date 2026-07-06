## 2024-07-06 - [CRITICAL] Fix SSRF in HTTP Client and SSE Connections
**Vulnerability:** The application was passing user-provided or integration-provided URLs directly to reqwest in execution.rs and sse.rs without validation, leading to potential Server-Side Request Forgery (SSRF) vulnerabilities.
**Learning:** External or templated URLs must always be validated before network requests, even when generated from internal system connections.
**Prevention:** Always use ferroflux_security::network::validate_url for any dynamically constructed URL used in outbound requests.
