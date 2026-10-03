## 2024-10-03 - SSRF IPv4-mapped IPv6 Bypass
**Vulnerability:** The SSRF protection mechanism in `is_blocked_ip` (in `ferroflux-security/src/network.rs`) failed to correctly identify IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`) as blocked. The standard `IpAddr::is_loopback()` returns false for these mapped IPv6 addresses, and the V6 matching arm lacks checking for them, allowing a bypass of SSRF protections.
**Learning:** In Rust, an `IpAddr::V6` address representing an IPv4-mapped address must be explicitly converted using `Ipv6Addr::to_ipv4_mapped()` before applying standard IPv4 range checks or loopback checks.
**Prevention:** Always extract and validate the mapped IPv4 address for IPv6 addresses before falling back to native IPv6 range checks.
