## 2024-09-24 - [HIGH] Fix SSRF bypass via IPv4-mapped IPv6
**Vulnerability:** The SSRF protection failed to properly map and block IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`), allowing an attacker to request internal network resources.
**Learning:** Checking for loopback and private IPs must correctly handle IPv4-mapped IPv6 addresses, as network stacks can resolve and connect to these addresses, bypassing simple string/IPv4 matching.
**Prevention:** Always unmap IPv6 addresses to IPv4 using `to_ipv4_mapped()` before applying standard IPv4 range checks when preventing SSRF.
