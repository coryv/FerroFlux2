## 2024-10-06 - [High] SSRF via IPv4-mapped IPv6 addresses bypass
**Vulnerability:** The application was not translating IPv4-mapped IPv6 addresses to their IPv4 equivalents before applying network blocking rules in `is_blocked_ip`. This allowed attackers to bypass private IP blocklists by providing URLs like `http://[::ffff:127.0.0.1]`.
**Learning:** Rust's standard IP range checks do not automatically treat IPv4-mapped IPv6 addresses as their IPv4 equivalents. Without explicit `.to_ipv4_mapped()` conversion, these addresses bypass standard IPv4 blocklists.
**Prevention:** Always unmap IPv4-mapped IPv6 addresses and re-check them against standard IPv4 restrictions.
