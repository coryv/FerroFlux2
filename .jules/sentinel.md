## 2025-02-12 - Prevent SSRF Bypass via IPv4-mapped IPv6
**Vulnerability:** The SSRF protection logic in `ferroflux-security` could be bypassed by supplying an IPv4-mapped IPv6 address (e.g., `::ffff:127.0.0.1`). The `is_blocked_ip` function checked IPv6 blocks without first unmapping the embedded IPv4 address, causing the standard IPv4 loopback/private checks to be skipped.
**Learning:** Standard IPv4 range checks fail to identify mapped local IPs if they are not explicitly unmapped first.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying range checks.
