## 2024-05-20 - [CRITICAL] Fix SSRF Bypass via IPv4-mapped IPv6 Address
**Vulnerability:** The `is_blocked_ip` function checks IPv4 loopback and private networks, and unique local/link-local IPv6 addresses, but did not handle IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`). Standard IPv6 range checks miss them because their initial segment is not `0xfc00` or `0xfe80`, allowing an attacker to reach internal services.
**Learning:** Rust's standard `is_loopback()` on `IpAddr` does not automatically classify `::ffff:127.0.0.1` as loopback on all platforms unless explicitly converted to IPv4.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses using `IpAddr::V6(ipv6).to_ipv4_mapped()` before applying IP address range checks or validations.
