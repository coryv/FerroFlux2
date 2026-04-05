use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use uuid::Uuid;
use hex;

pub struct CryptoTool;

impl Tool for CryptoTool {
    fn id(&self) -> &'static str {
        "core.utils.crypto"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let operation = params.get("operation").and_then(|v| v.as_str()).unwrap_or("hash");

        match operation {
            "hash" => {
                let input = params.get("input").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'input'"))?;
                let mut hasher = Sha256::new();
                hasher.update(input.as_bytes());
                let result = hasher.finalize();
                Ok(json!({ "result": hex::encode(result) }))
            },
            "hmac" => {
                let input = params.get("input").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'input'"))?;
                let key = params.get("key").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'key'"))?;
                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())?;
                mac.update(input.as_bytes());
                let result = mac.finalize();
                Ok(json!({ "result": hex::encode(result.into_bytes()) }))
            },
            "uuid" => {
                Ok(json!({ "result": Uuid::new_v4().to_string() }))
            },
            _ => Err(anyhow!("Unsupported operation: {}", operation)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferroflux_types::tool::ToolContext;
    use std::collections::HashMap;

    #[test]
    fn test_hash() {
        let tool = CryptoTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = ToolContext {
            local: &mut local,
            memory: &mut memory,
            trace_id: "test".to_string(),
            node_id: "test_node".to_string(),
            tenant_id: "test_tenant".to_string(),
            event_bus: None,
            shadow_mode: false,
            shadow_masks: &masks,
            store: None,
            secrets: None,
        };
        let params = json!({
            "operation": "hash",
            "input": "hello"
        });
        let res = tool.run(&mut ctx, params).unwrap();
        assert_eq!(res["result"], "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_uuid() {
        let tool = CryptoTool;
        let mut local = HashMap::new();
        let mut memory = HashMap::new();
        let masks = HashMap::new();
        let mut ctx = ToolContext {
            local: &mut local,
            memory: &mut memory,
            trace_id: "test".to_string(),
            node_id: "test_node".to_string(),
            tenant_id: "test_tenant".to_string(),
            event_bus: None,
            shadow_mode: false,
            shadow_masks: &masks,
            store: None,
            secrets: None,
        };
        let params = json!({ "operation": "uuid" });
        let res = tool.run(&mut ctx, params).unwrap();
        assert!(Uuid::parse_str(res["result"].as_str().unwrap()).is_ok());
    }
}
