## 2025-02-28 - Missing URL validation in SSE Trigger node
**Vulnerability:** The SSE Trigger node (`sse_trigger_system` in `crates/ferroflux-connectors/src/systems/sse.rs`) connected to arbitrary user-supplied URLs without validation.
**Learning:** This exposes the application to Server-Side Request Forgery (SSRF). Attackers could connect to internal networks or loopback interfaces via SSE streams. The RSS node properly checked `ferroflux_security::network::validate_url(&url)` but SSE missed it.
**Prevention:** Ensure that all external network connections utilizing user-provided URLs validate against `ferroflux_security::network::validate_url(&url)` or `check_ssrf(url)` prior to connection.
