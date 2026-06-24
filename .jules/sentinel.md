## 2025-02-14 - Fix SSRF Vulnerability in SSE Connector
**Vulnerability:** The SSE connector (`ferroflux-connectors/src/systems/sse.rs`) connected to external SSE streams without validating if the URL pointed to an internal IP.
**Learning:** In a connector-based system that fetches URLs, we must validate every external network boundary for SSRF.
**Prevention:** Use `ferroflux_security::network::validate_url` before making HTTP calls, especially inside background tasks.
