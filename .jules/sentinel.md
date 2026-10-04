## 2024-10-04 - SSRF Bypass via IPv4-Mapped IPv6 Addresses
**Vulnerability:** The URL validation logic failed to unmap IPv4-mapped IPv6 addresses (e.g., `::ffff:127.0.0.1`) before applying range checks. This allowed attackers to bypass internal blocklists and hit local or private network services.
**Learning:** Always extract the embedded IPv4 address from an IPv6 mapping and re-apply standard IPv4 range checks to prevent evasion tactics.
**Prevention:** Use `.to_ipv4_mapped()` on IPv6 addresses before evaluating them against blocklists.
