## 2026-09-11 - [Fix SSRF bypass via IPv4-mapped IPv6 addresses]
**Vulnerability:** SSRF filter bypass using IPv4-mapped IPv6 addresses (e.g., ::ffff:127.0.0.1).
**Learning:** Rust's standard IPv4 checks do not automatically apply to IPv4-mapped IPv6 addresses. A malicious user could provide ::ffff:127.0.0.1 to bypass local IP restrictions.
**Prevention:** Always explicitly unmap IPv4-mapped IPv6 addresses using Ipv6Addr::to_ipv4_mapped() before applying IP range checks in SSRF protections.
