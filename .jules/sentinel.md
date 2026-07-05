## 2024-05-31 - SSRF Vulnerability in GraphQL Tool
**Vulnerability:** The `core.utils.graphql` tool did not perform any validation on the requested `url` to prevent server-side request forgery (SSRF). This allowed potential attackers to make requests to internal network resources.
**Learning:** `reqwest::blocking::Client::new().post(url)` does not validate network targets by default. Other tools like `core.utils.http` use `crate::primitives::request::check_ssrf` to block internal IP requests (e.g. `127.0.0.1`, `10.x.x.x`), but this primitive missed it.
**Prevention:** Always wrap external HTTP requests with `ferroflux_security::network::validate_url` or `crate::primitives::request::check_ssrf` to filter out internal or sensitive hostnames/IPs before dialing.
