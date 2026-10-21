## 2025-02-18 - SSRF Bypass via IPv4-mapped IPv6 Address
**Vulnerability:** The SSRF protection logic (`is_blocked_ip`) successfully blocked dangerous IPv4 ranges but failed to check mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`).
**Learning:** Native IPv4 range checks fail to identify mapped IPs unless explicitly extracted/unmapped via `.to_ipv4_mapped()`. This is a common bypass for custom blocklists.
**Prevention:** Always explicitly unmap IPv4-mapped IPv6 addresses to standard IPv4 addresses using `.to_ipv4_mapped()` before applying standard network range filters.
