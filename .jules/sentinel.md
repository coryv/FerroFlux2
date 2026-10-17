## 2024-10-17 - [SSRF Bypass via IPv4-mapped IPv6 addresses]
**Vulnerability:** The `is_blocked_ip` function in `crates/ferroflux-security/src/network.rs` was vulnerable to Server-Side Request Forgery (SSRF) bypass because it did not check for IPv4-mapped IPv6 addresses.
**Learning:** Standard library `IpAddr` checks (`is_loopback()`) do not cover mapped IPv6 addresses if they are parsed as V6, even if they represent V4 IPs. This requires explicit unmapping before applying filters.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses using `ipv6.to_ipv4_mapped()` before applying standard IP blocklist checks to prevent loopback bypasses like `::ffff:127.0.0.1`.
