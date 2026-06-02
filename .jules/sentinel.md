## 2025-06-02 - Add SSRF Protection to GraphQL Tool
**Vulnerability:** The GraphQL tool did not implement SSRF protection, allowing requests to internal network IP addresses.
**Learning:** Network request primitive tools must always incorporate `check_ssrf` to validate URLs before making external requests, especially when allowing arbitrary URLs.
**Prevention:** Always use `check_ssrf` or `ferroflux_security::network::validate_url` for any network call where the user defines the target URL.
