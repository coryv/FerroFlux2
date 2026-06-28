## 2024-05-28 - Missing SSRF Validation in SSE Trigger
**Vulnerability:** The SSE trigger system in `ferroflux-connectors` did not validate the URL before establishing a connection via `reqwest`.
**Learning:** Background systems establishing long-lived connections (like SSE streams) need the same SSRF validations as immediate HTTP triggers.
**Prevention:** Always use `ferroflux_security::network::validate_url` before instantiating HTTP clients or sending requests with user-supplied URLs.
