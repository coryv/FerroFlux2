## 2025-02-23 - [CRITICAL] Fix SSRF vulnerability in tools
**Vulnerability:** The HTTP client tool relied on naive string matching for SSRF protection, which could be bypassed using alternative IP formats like IPv4-mapped IPv6 addresses or decimal IPs.
**Learning:** Naive string matching on URLs is insufficient for robust SSRF protection; domain resolution and exact IP classification are required.
**Prevention:** Always use the robust `ferroflux_security::network::validate_url` utility to protect against SSRF rather than ad-hoc string checks.
