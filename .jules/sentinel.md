## 2024-09-01 - SSRF Vulnerability in SSE Connections
**Vulnerability:** The SSE trigger system was making outbound requests to user-provided URLs using `reqwest` without any URL validation, which creates an SSRF risk for the application since it could reach internal endpoints.
**Learning:** Systems that perform background polling/streaming (like RSS and SSE workers) can often overlook SSRF checks, because the URL originates from internal configuration struct passing rather than direct HTTP input processing.
**Prevention:** Always use `ferroflux_security::network::validate_url` for any outbound request logic, especially in background workers handling user-defined connection configurations.
