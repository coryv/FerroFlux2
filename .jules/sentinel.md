
## 2024-02-09 - [CRITICAL] SSRF Bypass via IPv4-mapped IPv6 Addresses
**Vulnerability:** The application's SSRF protection (`validate_url`) could be bypassed by using IPv4-mapped IPv6 addresses like `http://[::ffff:127.0.0.1]`. Furthermore, `Url::host_str()` retained brackets for IPv6, preventing `to_socket_addrs()` from resolving the address entirely, thereby failing open to subsequent requests.
**Learning:** `std::net::ToSocketAddrs` fails to parse IPv6 hosts if brackets are included. Additionally, IPv4-mapped IPv6 addresses are treated as `IpAddr::V6` by the standard library but must be manually unwrapped using `.to_ipv4_mapped()` before applying standard IPv4 range blocklists, otherwise standard IPv4 blocking will fail to match them.
**Prevention:** Always strip brackets from URL hosts before DNS resolution. Explicitly unwrap `IpAddr::V6` mapping via `to_ipv4_mapped()` and check the resultant IPv4 address against blocklists before trusting any resolved IP.
