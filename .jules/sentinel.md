
## 2024-10-13 - SSRF Bypass via IPv4-Mapped IPv6 Addresses
**Vulnerability:** The SSRF blocklist in `network::is_blocked_ip` correctly blocked V4 loopbacks and private networks, but allowed V6 IPv4-mapped addresses (like `::ffff:127.0.0.1`), allowing attackers to bypass SSRF protections and access internal V4 networks by presenting requests disguised as V6.
**Learning:** `std::net::IpAddr` distinguishes between V4 and V6 types, so a custom `is_blocked_ip` range check will miss mapped IPs unless explicitly unmapped. `to_ipv4_mapped()` must always be called before checking ranges for network security functions.
**Prevention:** Always normalize IPs by unmapping IPv4-mapped IPv6 addresses at the top of IP validation functions.
