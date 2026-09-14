## 2024-10-14 - SSRF Bypass via IPv4-mapped IPv6 Addresses
**Vulnerability:** The SSRF protection in `ferroflux-security::network::is_blocked_ip` parses IP addresses and checks them against blocked ranges. However, it fails to unmap IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`) before matching against the IPv6 logic. A malicious actor could supply a mapped address to bypass the IPv4 internal/loopback checks.
**Learning:** Rust's standard library `IpAddr` allows representation of IPv4-mapped addresses via `IpAddr::V6`, which standard IPv4 checks do not catch unless explicitly unmapped.
**Prevention:** Always explicitly check for and unmap IPv4-mapped IPv6 addresses using `.to_ipv4_mapped()` before applying range checks in SSRF defense mechanisms.
