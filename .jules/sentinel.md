## 2024-05-18 - Fix SSRF via DNS Rebinding and missing URL validations
**Vulnerability:** The HTTP tool had a custom SSRF checker that only did basic string matching (blocking `localhost`, `127.0.0.1`, `10.*`), which was easily bypassable via IPv6 `[::1]`, hex/octal encoded IPs (`0x7f000001`), or DNS rebinding. Additionally, the GraphQL tool completely missed the URL validation.
**Learning:** `ferroflux_security::network::validate_url` exists and performs secure DNS resolution validation and is robust against alternative IP representations.
**Prevention:** Always use `ferroflux_security::network::validate_url` for user-provided URLs in HTTP/GraphQL actions to securely prevent Server-Side Request Forgery instead of relying on regex/string blocklists.
