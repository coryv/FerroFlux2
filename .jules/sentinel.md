## 2024-05-18 - [CRITICAL] SSRF bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The IP validation logic allowed IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`) to bypass restrictions meant for internal/private IPv4 networks, leading to a Server-Side Request Forgery (SSRF) bypass.
**Learning:** `IpAddr::is_loopback()` returns `false` for IPv4-mapped loopback addresses. Explicitly unmapping these addresses is required before applying IPv4 range checks to prevent bypasses.
**Prevention:** Always explicitly check for and unmap IPv4-mapped IPv6 addresses using `ipv6.to_ipv4_mapped()` before applying custom block lists or range checks in Rust.
