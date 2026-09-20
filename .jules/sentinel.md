
## 2024-09-20 - [Fix SSRF bypass via IPv4-mapped IPv6]
**Vulnerability:** The URL validation logic `validate_url` in `ferroflux-security` checked standard IP addresses to prevent SSRF vulnerabilities by matching loopback and private networks. However, it failed to resolve IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`), allowing an attacker to bypass standard IPv4 checks to hit local services.
**Learning:** SSRF protections must explicitly unmap IPv4-mapped IPv6 addresses to IPv4 using `Ipv6Addr::to_ipv4_mapped()` before applying range checks.
**Prevention:** Always convert IPv6 to IPv4 if it's an IPv4-mapped IPv6 address when validating IPs for SSRF protections.
