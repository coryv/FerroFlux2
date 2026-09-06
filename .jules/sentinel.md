## 2025-05-18 - Prevent SSRF Bypass with IPv4-Mapped IPv6
**Vulnerability:** The SSRF protection in `ferroflux-tools` used naive string matching against common localhost representations and private subnets. This allows bypass via DNS rebinding and obfuscated/mapped IPs like `::ffff:127.0.0.1`.
**Learning:** Checking for SSRF must involve proper hostname parsing, IP resolution, and inspecting the actual resolved IP instead of raw URL string tokens.
**Prevention:** Always use dedicated URL/network validation utilities (like `ferroflux_security::network::validate_url`) that resolve IPs and apply comprehensive blocking rules to prevent obscure IP formatting bypasses.
