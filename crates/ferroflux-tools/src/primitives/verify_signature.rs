//! Webhook Signature Verification Tool
//!
//! Verifies that an inbound webhook request was signed by the expected platform
//! using HMAC-SHA256 or HMAC-SHA1. Place this as the first step after a
//! `trigger.webhook` node to reject unauthenticated requests before any
//! business logic runs.
//!
//! On mismatch, returns `Err(...)` which routes the pipeline to its Error output.
//! On success, returns `{ "verified": true }`.

use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::Sha256;
use serde_json::Value;

type HmacSha256 = Hmac<Sha256>;
type HmacSha1 = Hmac<Sha1>;

pub struct VerifySignatureTool;

impl Tool for VerifySignatureTool {
    fn id(&self) -> &'static str {
        "verify_signature"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let body = params
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'body' param"))?;

        let secret = params
            .get("secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'secret' param"))?;

        let raw_signature = params
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'signature' param"))?;

        let algorithm = params
            .get("algorithm")
            .and_then(|v| v.as_str())
            .unwrap_or("hmac-sha256");

        let encoding = params
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("hex");

        // Strip algorithm prefix (e.g. "sha256=", "v0=") — prefix must be ≤10 chars
        let signature = strip_algorithm_prefix(raw_signature);

        // Decode the expected signature bytes
        let expected_bytes = match encoding {
            "base64" => general_purpose::STANDARD
                .decode(signature)
                .context("Failed to base64-decode signature")?,
            _ => hex::decode(signature).context("Failed to hex-decode signature")?,
        };

        // Compute HMAC and compare (constant-time)
        let verified = match algorithm {
            "hmac-sha1" => {
                let mut mac = HmacSha1::new_from_slice(secret.as_bytes())
                    .context("Invalid HMAC-SHA1 key length")?;
                mac.update(body.as_bytes());
                mac.verify_slice(&expected_bytes).is_ok()
            }
            _ => {
                // Default: hmac-sha256
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .context("Invalid HMAC-SHA256 key length")?;
                mac.update(body.as_bytes());
                mac.verify_slice(&expected_bytes).is_ok()
            }
        };

        if verified {
            Ok(serde_json::json!({ "verified": true }))
        } else {
            Err(anyhow!(
                "Webhook signature verification failed: HMAC mismatch"
            ))
        }
    }
}

/// Strips a short algorithm prefix (e.g. `sha256=`, `sha1=`, `v0=`) from a
/// signature string. The prefix must be at most 10 characters and end with `=`.
fn strip_algorithm_prefix(sig: &str) -> &str {
    if let Some(eq_pos) = sig.find('=') && eq_pos <= 10 {
        return &sig[eq_pos + 1..];
    }
    sig
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    // Pre-compute expected signatures using the same crates for test vectors.
    fn hmac_sha256_hex(secret: &str, body: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn hmac_sha1_hex(secret: &str, body: &str) -> String {
        let mut mac = HmacSha1::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn hmac_sha256_b64(secret: &str, body: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        general_purpose::STANDARD.encode(mac.finalize().into_bytes())
    }

    macro_rules! make_ctx {
        () => {{
            let local = Box::leak(Box::new(HashMap::new()));
            let memory = Box::leak(Box::new(HashMap::new()));
            let masks = Box::leak(Box::new(HashMap::new()));
            ToolContext {
                local,
                memory,
                trace_id: "test".to_string(),
                node_id: "test_node".to_string(),
                tenant_id: "test_tenant".to_string(),
                event_bus: None,
                shadow_mode: false,
                shadow_masks: masks,
                store: None,
                secrets: None,
            }
        }};
    }

    #[test]
    fn test_hmac_sha256_hex_ok() {
        let secret = "my_secret";
        let body = r#"{"id":1,"event":"push"}"#;
        let sig = hmac_sha256_hex(secret, body);

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": body,
            "secret": secret,
            "signature": sig,
            "algorithm": "hmac-sha256",
            "encoding": "hex"
        });
        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["verified"], true);
    }

    #[test]
    fn test_hmac_sha1_hex_ok() {
        let secret = "legacy_secret";
        let body = r#"{"event":"ping"}"#;
        let sig = hmac_sha1_hex(secret, body);

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": body,
            "secret": secret,
            "signature": sig,
            "algorithm": "hmac-sha1",
            "encoding": "hex"
        });
        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["verified"], true);
    }

    #[test]
    fn test_base64_encoding_ok() {
        let secret = "base64_secret";
        let body = r#"{"amount":100}"#;
        let sig = hmac_sha256_b64(secret, body);

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": body,
            "secret": secret,
            "signature": sig,
            "algorithm": "hmac-sha256",
            "encoding": "base64"
        });
        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["verified"], true);
    }

    #[test]
    fn test_prefix_stripping_github_style() {
        let secret = "github_secret";
        let body = r#"{"ref":"refs/heads/main"}"#;
        let raw_sig = format!("sha256={}", hmac_sha256_hex(secret, body));

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": body,
            "secret": secret,
            "signature": raw_sig,
            "algorithm": "hmac-sha256",
            "encoding": "hex"
        });
        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["verified"], true);
    }

    #[test]
    fn test_prefix_stripping_slack_style() {
        let secret = "slack_signing_secret";
        let body = "v0:1234567890:payload=here";
        let raw_sig = format!("v0={}", hmac_sha256_hex(secret, body));

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": body,
            "secret": secret,
            "signature": raw_sig,
            "algorithm": "hmac-sha256",
            "encoding": "hex"
        });
        let result = tool.run(&mut ctx, params).unwrap();
        assert_eq!(result["verified"], true);
    }

    #[test]
    fn test_wrong_secret_fails() {
        let body = r#"{"event":"payment"}"#;
        let sig = hmac_sha256_hex("correct_secret", body);

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": body,
            "secret": "wrong_secret",
            "signature": sig,
            "algorithm": "hmac-sha256",
            "encoding": "hex"
        });
        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(err.to_string().contains("verification failed"), "{}", err);
    }

    #[test]
    fn test_tampered_body_fails() {
        let secret = "my_secret";
        let original_body = r#"{"amount":100}"#;
        let tampered_body = r#"{"amount":9999}"#;
        let sig = hmac_sha256_hex(secret, original_body);

        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": tampered_body,
            "secret": secret,
            "signature": sig,
            "algorithm": "hmac-sha256",
            "encoding": "hex"
        });
        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(err.to_string().contains("verification failed"), "{}", err);
    }

    #[test]
    fn test_invalid_hex_returns_error() {
        let tool = VerifySignatureTool;
        let mut ctx = make_ctx!();
        let params = serde_json::json!({
            "body": "some body",
            "secret": "secret",
            "signature": "not-valid-hex!!!",
            "algorithm": "hmac-sha256",
            "encoding": "hex"
        });
        let err = tool.run(&mut ctx, params).unwrap_err();
        assert!(err.to_string().contains("hex-decode"), "{}", err);
    }
}
