## 2024-10-08 - SSRF Vulnerability via Dynamic URL Construction
**Vulnerability:** Core execution engine dynamically constructs URLs based on user inputs/templates and uses `reqwest` without validating against private/local networks.
**Learning:** External API integrations must always wrap URL resolution with network validation to prevent SSRF.
**Prevention:** Use `ferroflux_security::network::validate_url` for any network call that uses dynamically constructed endpoints.
