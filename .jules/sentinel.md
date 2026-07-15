## 2024-05-01 - [API Key Entropy Enhancement]
**Vulnerability:** API key generation used UUIDv4 (122-bit entropy). While cryptographically secure (uses `getrandom`), a 256-bit key is the preferred industry standard for high-security environments against brute-force attacks.
**Learning:** `uuid::Uuid::new_v4()` provides 122 bits of entropy, which is secure but falls short of the 256-bit standard often desired for API keys.
**Prevention:** Use `rand::rngs::OsRng` to generate 256-bit (32-byte) keys and encode them with `hex::encode` to provide stronger cryptographic guarantees for API keys.
