use anyhow::{anyhow, Result, Context};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// A payload that can be signed to verify the integrity and authorship of an integration.
/// 
/// For YAML integrations, we hash the entire normalized content (excluding the signature field itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSignature {
    pub public_key: String, // Hex encoded
    pub signature: String,  // Hex encoded
    pub signer_name: String,
}

/// Signs a canonical representation of an integration's YAML content.
pub fn sign_content(content: &serde_json::Value, private_key_hex: &str) -> Result<IntegrationSignature> {
    let key_bytes = hex::decode(private_key_hex).context("Invalid private key hex")?;
    if key_bytes.len() != 32 {
        return Err(anyhow!("Private key must be 32 bytes"));
    }
    
    let signing_key = SigningKey::from_bytes(key_bytes.as_slice().try_into()?);
    let verifying_key = signing_key.verifying_key();
    
    // Canonicalize the JSON to ensure stable hashing
    let canonical_bytes = serde_json::to_vec(content)?;
    
    let signature = signing_key.sign(&canonical_bytes);
    
    Ok(IntegrationSignature {
        public_key: hex::encode(verifying_key.to_bytes()),
        signature: hex::encode(signature.to_bytes()),
        signer_name: "FerroFlux Developer".to_string(), // In practice, this would be retrieved from profile
    })
}

/// Verifies that the provided content matches the cryptographic signature.
pub fn verify_content(content: &serde_json::Value, signature_info: &IntegrationSignature) -> Result<()> {
    let pub_key_bytes = hex::decode(&signature_info.public_key).context("Invalid public key hex")?;
    let sig_bytes = hex::decode(&signature_info.signature).context("Invalid signature hex")?;
    
    let verifying_key = VerifyingKey::from_bytes(pub_key_bytes.as_slice().try_into()?)
        .map_err(|e| anyhow!("Invalid public key: {}", e))?;
    
    let signature = Signature::from_bytes(sig_bytes.as_slice().try_into()?);
    
    // Canonicalize the JSON
    let canonical_bytes = serde_json::to_vec(content)?;
    
    verifying_key.verify(&canonical_bytes, &signature)
        .map_err(|e| anyhow!("Signature verification failed: {}", e))?;
    
    Ok(())
}

/// A trusted registry of public keys allowed to sign "Official" or "Verified" integrations.
pub fn is_trusted_key(public_key_hex: &str) -> bool {
    // In a production system, this would be a dynamic list or baked-in roots of trust.
    const OFFICIAL_ROOT: &str = "0000000000000000000000000000000000000000000000000000000000000000"; // Placeholder
    public_key_hex == OFFICIAL_ROOT
}
