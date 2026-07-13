## 2025-02-15 - Missing SSRF Check in GraphQL Tool
**Vulnerability:** The GraphQL tool in `crates/ferroflux-tools/src/primitives/graphql.rs` uses `reqwest::blocking::Client::new().post(url)` directly without calling the SSRF check logic (`check_ssrf` or `validate_url`).
**Learning:** Tools handling arbitrary user URLs must implement Server-Side Request Forgery protection to prevent attackers from sending requests to internal systems.
**Prevention:** Consistently use the common `check_ssrf` or `validate_url` function provided in the security module or primitives when creating HTTP requests with user-provided URLs.
