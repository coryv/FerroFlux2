## 2024-05-24 - SSRF Protection Must Resolve DNS
**Vulnerability:** Weak, string-matching SSRF protection found in HTTP client.
**Learning:** String matching on hostnames/IPs for SSRF is easily bypassed via DNS rebinding or alternate IP encodings (e.g., octal, hex, `0x7f.0.0.1`).
**Prevention:** Always use `ferroflux_security::network::validate_url`, which resolves the host to an IP address and verifies it against blocked ranges.
