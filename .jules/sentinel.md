## 2024-05-24 - Fix SSRF bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The URL validation logic did not handle IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`), allowing SSRF bypasses by reaching internal loops.
**Learning:** Always unmap IPv6 addresses to IPv4 if they are IPv4-mapped before applying range checks.
**Prevention:** Use `to_ipv4_mapped()` on IPv6 addresses before validating IPs.
