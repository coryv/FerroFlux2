## 2024-10-07 - [SSRF Bypass via IPv4-mapped IPv6 Address]
**Vulnerability:** The SSRF filter checking IPv4 private and loopback IP addresses in `ferroflux-security` could be bypassed by using IPv4-mapped IPv6 addresses like `::ffff:127.0.0.1`.
**Learning:** IPv4 addresses can be mapped inside IPv6 ranges and need explicit unwrapping when validating. The default Rust `is_loopback` method doesn't capture V4 mapped localhosts from V6 objects properly either.
**Prevention:** In SSRF validation logics, explicitly check for and convert mapped IPv4s in IPv6 inputs `if let Some(ipv4) = ipv6.to_ipv4_mapped()` before applying range checks.
