## 2023-08-06 - Initial setup
**Vulnerability:** N/A
**Learning:** N/A
**Prevention:** N/A
## 2023-08-06 - Enforce strict IP resolution for SSRF
**Vulnerability:** Naive string matching for SSRF protection in HTTP client tool could be bypassed via DNS resolution or alternative IP formats (e.g., 0x7f.0.0.1).
**Learning:** Always use `ferroflux_security::network::validate_url` which resolves hostnames to IPs and explicitly checks against internal ranges (loopback, private, link-local).
**Prevention:** Rely on standard internal security primitives for network validation rather than ad-hoc string checks.
