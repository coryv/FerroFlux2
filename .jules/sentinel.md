## 2024-05-24 - Fix SSRF Vulnerability in HTTP Request Client
**Vulnerability:** Naive string matching in `check_ssrf` allowed bypassing Server-Side Request Forgery protections by using alternative representations of localhost (e.g. `0x7f.0.0.1`) or other internal IPs.
**Learning:** String matching on a URL host string is fundamentally insecure for SSRF protection because attackers can construct hosts that parse differently across DNS/IP routing layers.
**Prevention:** Always validate URLs by resolving the host to an IP address and checking that the resolved IP against blocked, private or loopback ranges, as implemented in `ferroflux_security::network::validate_url`.
