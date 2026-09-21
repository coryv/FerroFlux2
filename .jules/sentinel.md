## 2024-05-24 - Fix SSRF bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The URL validation logic failed to unmap IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`) before evaluating them against blocked IPv6 ranges, allowing requests to private/loopback IPv4 addresses.
**Learning:** In Rust, `IpAddr::is_loopback()` and standard range checks don't automatically unwrap mapped IPv6 addresses. If an address can be mapped down, it bypasses the `V6` branch checks.
**Prevention:** Always explicitly check for and unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying range validation.
