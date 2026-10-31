## 2023-11-01 - [Server-Side Request Forgery via IPv4-mapped IPv6]
**Vulnerability:** The `is_blocked_ip` function in `crates/ferroflux-security/src/network.rs` lacks explicit unmapping of IPv4-mapped IPv6 addresses, which allows a malicious user to bypass standard IPv4 blocklists by requesting addresses like `::ffff:127.0.0.1` and achieve Server-Side Request Forgery (SSRF).
**Learning:** `IpAddr::V6(ipv6)` standard resolution does not inherently apply IPv4 constraints to mapped IPv6 representations. It must be manually converted and then tested against the IPv4 rules.
**Prevention:** Explicitly unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying range checks.
