## 2024-07-01 - 🛡️ Sentinel: [CRITICAL] Fix weak magic link tokens
**Vulnerability:** Magic links were generated using `Uuid::new_v4().to_string()` which does not provide sufficient entropy for cryptographically secure auth tokens.
**Learning:** For generating cryptographically secure authentication tokens (like magic links), the codebase favors using 256-bit (32-byte) random values from `rand::rngs::OsRng` hex-encoded via the `hex` crate.
**Prevention:** Avoid `Uuid::new_v4()` for auth tokens; use `rand::rngs::OsRng` and `hex::encode` instead.
