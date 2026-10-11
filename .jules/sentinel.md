## 2024-05-24 - SSRF Bypass via IPv4-mapped IPv6 addresses
**Vulnerability:** The application's SSRF protection missed IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`) and failed to correctly parse URL hosts containing brackets for IPv6. This allowed bypassing checks and reaching internal loopback or private ranges.
**Learning:** `std::net::Ipv6Addr` does not automatically evaluate to its mapped IPv4 equivalent when doing range checks. Also, `Url::host_str()` preserves brackets for IPv6 addresses which causes `to_socket_addrs()` to fail or return nothing.
**Prevention:** Explicitly check for IPv4-mapped addresses via `Ipv6Addr::to_ipv4_mapped()` before doing security range checks. Additionally, trim brackets from IPv6 host strings before doing socket resolution.
