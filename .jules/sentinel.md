## 2024-05-24 - SSRF Bypass via IPv4-mapped IPv6 Addresses
**Vulnerability:** The SSRF check was validating IPv4 addresses correctly but failed to check if an IPv6 address was an IPv4-mapped address (e.g. `::ffff:127.0.0.1`). This allows an attacker to bypass IPv4 blocking rules by representing blocked IPv4 addresses in their IPv6-mapped format.
**Learning:** The URL parser preserves IPv4-mapped IPv6 addresses as IPv6, and standard `IpAddr::is_loopback()` or range checking logic does not translate them back to IPv4.
**Prevention:** Always explicitly check for and unmap IPv4-mapped IPv6 addresses (using `IpAddr::to_ipv4_mapped()`) before applying network restriction logic.
