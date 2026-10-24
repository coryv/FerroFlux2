## 2024-10-24 - SSRF bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The SSRF check failed to unmap IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`), allowing requests to bypass the loopback and internal IP blocklists since they were processed as V6, but ultimately routed as V4 by the OS.
**Learning:** Raw string matching and basic checks on `IpAddr` are insufficient if dual-stack mapped addresses aren't normalized.
**Prevention:** Always normalize `IpAddr::V6` addresses using `to_ipv4_mapped()` before applying SSRF/blocklist checks.
