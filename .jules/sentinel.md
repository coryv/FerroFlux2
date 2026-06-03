## 2024-06-03 - Missing SSRF Check in GraphQL Primitive
**Vulnerability:** The GraphQL primitive (`crates/ferroflux-tools/src/primitives/graphql.rs`) failed to validate target URLs, allowing Server-Side Request Forgery (SSRF) against internal services.
**Learning:** Network request tools must consistently apply the shared `check_ssrf` utility before executing requests, as arbitrary URLs are passed from user space.
**Prevention:** Always invoke `crate::primitives::request::check_ssrf` on user-provided URLs in any new network request primitive.
