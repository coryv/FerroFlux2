## 2025-01-01 - Fix SSRF Vulnerability in Tools
**Vulnerability:** The `check_ssrf` function in `ferroflux-tools/src/primitives/request.rs` relied on naïve string matching to block internal IP ranges. This is vulnerable to bypasses using IPv6 or other representations.
**Learning:** Always use a robust, DNS-resolving URL validator that blocks private IP spaces comprehensively.
**Prevention:** Use `ferroflux_security::network::validate_url` to properly resolve and block internal IP access in tools making external requests.
