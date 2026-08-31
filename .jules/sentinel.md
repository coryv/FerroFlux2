## 2026-08-20 - SSRF Protection via DNS Resolution
**Vulnerability:** `check_ssrf` relied on naive string matching of hostnames to block internal IP addresses (e.g., checking if it starts with "192.168.").
**Learning:** Naive string matching is easily bypassed by DNS rebinding or alternative IP representation.
**Prevention:** Always use a DNS-resolving network validation check, like `ferroflux_security::network::validate_url`, to evaluate the actual resolved IP against private IP space ranges.
