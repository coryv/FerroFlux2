## 2024-10-24 - [CRITICAL] Fix IPv4-Mapped IPv6 SSRF Bypass
**Vulnerability:** The URL validation logic did not block IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`), allowing an attacker to bypass SSRF protections and access internal resources.
**Learning:** Standard IPv4 range checks fail to identify IPv4-mapped IPv6 addresses, so they must be explicitly unmapped.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying range checks.
