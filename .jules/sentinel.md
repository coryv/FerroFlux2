## 2025-05-15 - [CRITICAL/HIGH] Fix SSRF in GraphQL tool
**Vulnerability:** Server-Side Request Forgery (SSRF) in the GraphQL primitive execution.
**Learning:** External user inputs were passed unvalidated to the HTTP client inside tools/GraphQL, allowing internal network scanning.
**Prevention:** Always use `crate::primitives::request::check_ssrf` on external URLs in primitives.
