## 2025-02-27 - Insecure Magic Link Generation
**Vulnerability:** The `create_magic_link` function in `crates/ferroflux-iam/src/lib.rs` uses standard `Uuid::new_v4()` to generate magic links.
**Learning:** For generating cryptographically secure authentication tokens (like magic links), the codebase favors using 256-bit (32-byte) random values from `rand::rngs::OsRng` hex-encoded via the `hex` crate, as this provides higher entropy than standard UUID v4.
**Prevention:** Always use `rand::rngs::OsRng` with `hex::encode` for generating authentication tokens and magic links to ensure sufficient cryptographic entropy.
