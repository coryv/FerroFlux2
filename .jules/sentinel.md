## 2024-10-30 - Fix SSRF IPv4-mapped IPv6 bypass
**Vulnerability:** URL validation bypass using IPv4-mapped IPv6 addresses (e.g. `[::ffff:127.0.0.1]`).
**Learning:** Rust's standard library `Url` and `ToSocketAddrs` allow IPv6 formats that embed IPv4 addresses, which can bypass basic IP validation if we only check `is_loopback()` directly on the `IpAddr`. Also, `Url::host_str()` retains the square brackets for IPv6 addresses, so they need to be trimmed before passing to `ToSocketAddrs`.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses using `Ipv6Addr::to_ipv4_mapped()` before applying range checks, and explicitly handle brackets for IPv6 hosts.
