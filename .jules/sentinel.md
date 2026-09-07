
## 2024-09-07 - [CRITICAL] Prevent SSRF bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The `is_blocked_ip` function in `ferroflux-security` failed to identify IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`), allowing attackers to bypass SSRF protections and access internal services (like `127.0.0.1` or cloud metadata at `169.254.169.254`).
**Learning:** Standard IPv4 range checks do not automatically unmap or recognize IPv4-mapped IPv6 addresses in Rust's `IpAddr`. Always explicitly check for and unmap mapped IPs before validating IP ranges for SSRF protection.
**Prevention:** Call `ip = IpAddr::V4(ipv4);` after `ipv6.to_ipv4_mapped()` before performing blocklist range checks.
