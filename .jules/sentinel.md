## 2024-07-30 - Sentinel Init
**Vulnerability:** Initial run
**Learning:** Initializing journal
**Prevention:** N/A
## 2024-07-30 - Fix SSRF Vulnerability in Tools
**Vulnerability:** Simple string-based SSRF check in `check_ssrf` bypasses many forms of SSRF attacks (e.g. 127.1, DNS rebinding to internal IPs, IPv6).
**Learning:** Naive string matching on URLs is insufficient to prevent SSRF vulnerabilities. Instead, the host must be resolved to its underlying IP and checked against a comprehensive list of blocked ranges.
**Prevention:** Use `ferroflux_security::network::validate_url` which properly resolves and checks IP address ranges.
