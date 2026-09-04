## 2024-09-04 - SSRF Bypass via IPv4-mapped IPv6 and Incomplete Pattern Matching
**Vulnerability:** Naive string matching for SSRF protection failed to cover IPv4-mapped IPv6 addresses (e.g. ::ffff:127.0.0.1) and omitted other internal ranges (e.g. link-local, unique local, 172.16.0.0/12).
**Learning:** Checking host strings directly is insufficient for SSRF protection. Attackers can bypass naive blocks using alternative representations of internal IPs (like mapped IPv6).
**Prevention:** Always resolve the host and rely on `IpAddr` checks with proper mapped IP handling, as implemented in `ferroflux_security::network::validate_url`.
