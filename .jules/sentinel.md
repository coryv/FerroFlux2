## 2024-11-03 - [SSRF Bypass via IPv4-mapped IPv6]
**Vulnerability:** The SSRF protection failed to identify blocked IPv4 addresses (like 127.0.0.1) when they were provided as IPv4-mapped IPv6 addresses (like ::ffff:127.0.0.1) because standard library socket resolution retained them as V6 types which did not match V4 block logic.
**Learning:** Rust's standard library `to_socket_addrs` does not automatically unmap IPv4-mapped IPv6 addresses. Explicit conversion via `Ipv6Addr::to_ipv4_mapped` is required before applying security range checks. Additionally, `url::Url` retains brackets for IPv6 hosts which must be trimmed before standard library DNS resolution.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses explicitly when validating network address ranges for SSRF protections.
