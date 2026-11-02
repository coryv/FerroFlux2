## 2024-11-02 - [SSRF Bypass via IPv4-Mapped IPv6 Addresses]
**Vulnerability:** The `validate_url` function blocked internal IPv4 ranges but failed to identify them when presented as IPv4-mapped IPv6 addresses (e.g. `[::ffff:127.0.0.1]`), allowing SSRF to local internal services.
**Learning:** `to_socket_addrs()` does not automatically normalize IPv4-mapped addresses to IPv4 enums. Furthermore, `Url::host_str()` retains IPv6 brackets (`[::1]`) which causes raw `to_socket_addrs()` to fail on IPv6 literals.
**Prevention:** Explicitly unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying range checks. Trim brackets from `url.host_str()` before socket resolution.
