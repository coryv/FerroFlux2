## 2024-10-27 - SSRF Protection via Centralized Security Library
**Vulnerability:** Naive string-prefix based SSRF protection in tools network client could be easily bypassed by IPv6 addresses, different encoding formats, or DNS rebinding.
**Learning:** Security validation functions for network requests must reliably parse, resolve, and check raw IPs rather than using simple string matching to properly block local services and metadata endpoints.
**Prevention:** Always use `ferroflux_security::network::validate_url` to handle complex IP resolution securely instead of implementing bespoke checks in individual modules.
