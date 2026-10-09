## 2024-05-24 - SSRF Bypass via IPv4-mapped IPv6
**Vulnerability:** The URL validation logic used to prevent Server-Side Request Forgery (SSRF) failed to properly identify blocked internal network addresses when passed as IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`).
**Learning:** Standard IPv4 range checks and standard IPv6 range checks do not automatically unmap or catch IPv4-mapped IPv6 local addresses in standard Rust `std::net::IpAddr` checks.
**Prevention:** Always explicitly unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying IP address range checks.
