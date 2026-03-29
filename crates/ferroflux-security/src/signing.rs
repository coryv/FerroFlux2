use anyhow::{anyhow, Result, Context};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// A payload that can be signed to verify the integrity and authorship of an integration.
/// 
/// For YAML integrations, we hash the entire normalized content (excluding the signature field itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSignature {
    pub key_version: String,
    pub public_key: String, // Hex encoded
    pub signature: String,  // Hex encoded
    pub signer_name: String,
}

/// Signs a canonical representation of an integration's YAML content.
pub fn sign_content(content: &serde_json::Value, private_key_hex: &str, key_version: &str) -> Result<IntegrationSignature> {
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
        key_version: key_version.to_string(),
        public_key: hex::encode(verifying_key.to_bytes()),
        signature: hex::encode(signature.to_bytes()),
        signer_name: "FerroFlux Official".to_string(),
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
pub fn is_trusted_key(public_key_hex: &str, version: &str) -> bool {
    // In production, this would be a map of versions to allowed public keys.
    match version {
        "v1" => public_key_hex == "0000000000000000000000000000000000000000000000000000000000000000", // Placeholder
        _ => false,
    }
}
