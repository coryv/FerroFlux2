## 2024-08-04 - Fix SSRF in GraphQL Tool
**Vulnerability:** The `GraphQlTool` in `ferroflux-tools` fetched URLs without checking if they were internal, posing an SSRF vulnerability.
**Learning:** All modules initiating outbound HTTP requests must explicitly perform SSRF validation, particularly since they may be developed distinctly from the generic HTTP client logic.
**Prevention:** Use the centralized `ferroflux_security::network::validate_url` or `check_ssrf` function for all network operations accepting arbitrary inputs.
