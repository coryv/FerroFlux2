## 2026-04-07 - [SSRF in Execution System]
**Vulnerability:** The HTTP execution system constructed a URL using user-supplied path strings via templates (`let url = format!("{}{}", def.base_url, path_str);`) and immediately passed it to `reqwest` without any SSRF validation.
**Learning:** Even though `def.base_url` might be trusted, the dynamically appended `path_str` can easily contain directory traversal characters (`../`) or point to an attacker-controlled endpoint that redirects to an internal network address, circumventing the intended base URL.
**Prevention:** Always use `ferroflux_security::network::validate_url` on the final constructed URL string *before* instantiating the HTTP client to explicitly resolve the IP and block internal/loopback ranges.
