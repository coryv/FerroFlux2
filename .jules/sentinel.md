## 2024-03-24 - SSRF Bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The SSRF protection `is_blocked_ip` in `ferroflux-security` correctly checks IPv4 local/private ranges, but fails to prevent SSRF bypasses via IPv4-mapped IPv6 addresses like `::ffff:127.0.0.1`.
**Learning:** Rust's `IpAddr` enum does not automatically normalize IPv4-mapped IPv6 addresses to `IpAddr::V4`. We must explicitly unmap them before applying range checks.
**Prevention:** Always explicitly check for and unmap IPv4-mapped IPv6 addresses using `.to_ipv4_mapped()` before applying custom security validation logic on `IpAddr` values.
