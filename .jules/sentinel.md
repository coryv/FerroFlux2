## 2024-05-18 - Missing SSRF Protection in GraphQL Tool
**Vulnerability:** The GraphQL tool in `ferroflux-tools` executes HTTP requests using a user-provided URL without any validation, allowing Server-Side Request Forgery (SSRF) attacks against internal services.
**Learning:** Tools that make network requests must explicitly validate URLs, as generic request primitives do not automatically apply security checks. The `check_ssrf` function exists but was overlooked when the new GraphQL tool was implemented.
**Prevention:** All components performing network requests must incorporate SSRF validation (e.g., `crate::primitives::request::check_ssrf` or `ferroflux_security::network::validate_url`). A global egress filter could provide defense-in-depth.
