## 2024-10-25 - Fix SSRF bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The application failed to identify and block internal IPv4 addresses if they were passed in their IPv4-mapped IPv6 format (e.g., `::ffff:127.0.0.1`).
**Learning:** `is_blocked_ip` in standard library or custom implementations will not apply standard IPv4 checks to IPv4-mapped IPv6 addresses unless they are explicitly mapped to IPv4 first.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` (e.g., after matching `IpAddr::V6(ipv6)`) before applying range checks.
