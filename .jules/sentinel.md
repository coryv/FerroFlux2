## 2025-02-27 - [CRITICAL] Fix SSRF bypass in request tool
**Vulnerability:** The request tool relied on naive string matching of the host (e.g., checking if it starts with "192.168." or is "127.0.0.1") to prevent SSRF. This can be easily bypassed using IPv6, mapped IPv4, integer IPs, or DNS resolving to internal IPs.
**Learning:** Never rely on string matching for SSRF protection. Always parse the URL, extract the host, resolve it to an IP address, and check if that IP falls within blocked ranges.
**Prevention:** Use the `ferroflux_security::network::validate_url` function to validate URLs, which properly resolves IPs and blocks internal ranges.
