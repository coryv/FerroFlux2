## 2025-02-27 - SSRF Protection via DNS Resolution
**Vulnerability:** Weak SSRF check in HTTP client (`crates/ferroflux-tools/src/primitives/request.rs`) only blocked simple strings (`localhost`, `10.`, etc.) and did not resolve hostnames, leaving it vulnerable to DNS rebinding, AWS metadata endpoints (`169.254.169.254`), and alternative IP encodings.
**Learning:** Hardcoded string checks on URLs are insufficient for SSRF protection because attackers can map domains to internal IPs or use different representations of internal IPs.
**Prevention:** Always use `ferroflux_security::network::validate_url` which parses the URL, performs DNS resolution, and checks the resolved IP addresses against a comprehensive list of blocked ranges.
