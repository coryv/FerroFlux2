use anyhow::{anyhow, Context, Result};
use clap::Parser;
use ferroflux_security::signing::sign_content;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// FerroFlux Integration Signing Tool
/// 
/// Signs an integration YAML file with a private key.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the integration YAML file to sign
    #[arg(short, long)]
    file: PathBuf,

    /// Hex-encoded 32-byte Ed25519 private key
    #[arg(short, long, env = "FERROFLUX_PRIVATE_KEY")]
    key: String,

    /// Version of the key being used (e.g. "v1")
    #[arg(short, long, default_value = "v1")]
    version: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Load YAML
    let yaml_str = fs::read_to_string(&args.file)
        .context(format!("Failed to read file: {:?}", args.file))?;
    let mut content: Value = serde_yaml::from_str(&yaml_str)
        .context("Failed to parse YAML")?;

    // 2. Remove existing signature from content if present (integrity)
    if let Some(obj) = content.as_object_mut() {
        if let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut()) {
            meta.remove("signature");
        }
    } else {
        return Err(anyhow!("Invalid YAML structure: missing root object"));
    }

    // 3. Sign the content
    let signature = sign_content(&content, &args.key, &args.version)?;

    // 4. Inject signature back into meta
    if let Some(obj) = content.as_object_mut()
        && let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut())
    {
        meta.insert("signature".to_string(), serde_json::to_value(signature)?);
    }

    // 5. Save back to YAML
    let updated_yaml = serde_yaml::to_string(&content)?;
    fs::write(&args.file, updated_yaml)
        .context(format!("Failed to write signed YAML to {:?}", args.file))?;

    println!("Successfully signed {:?}", args.file);
    Ok(())
}
