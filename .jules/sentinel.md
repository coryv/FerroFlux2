## 2024-10-14 - SSRF Bypass via IPv4-Mapped IPv6
**Vulnerability:** The validate_url function was vulnerable to Server-Side Request Forgery (SSRF) bypasses because it failed to resolve IPv6 addresses formatted with brackets from Url parsing (like '[::1]') and failed to identify private networks in IPv4-mapped IPv6 addresses (e.g. '::ffff:127.0.0.1').
**Learning:** Url::host_str() retains brackets for IPv6. Standard IP classification fails on mapped IPv4 addresses unless explicitly unmapped first.
**Prevention:** Strip brackets prior to DNS resolution, and unmap IPv4-mapped IPv6 addresses before enforcing security ranges.
