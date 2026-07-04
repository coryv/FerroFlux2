## 2024-05-18 - Fix SSRF in graphql primitive
**Vulnerability:** The GraphQL tool primitive in `ferroflux-tools` instantiated `reqwest::blocking::Client::new().post(url)` directly using user-provided URLs without first verifying that the URL was safe, creating a Server-Side Request Forgery (SSRF) vulnerability.
**Learning:** `ferroflux-tools` has its own internal SSRF protection wrapper `crate::primitives::request::check_ssrf(url)?` which must be called on every URL parameter before building HTTP requests.
**Prevention:** Ensure all network operations in tools and primitives manually call `check_ssrf` or `ferroflux_security::network::validate_url` before executing HTTP requests.
