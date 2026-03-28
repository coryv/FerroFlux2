use anyhow::Result;
use ferroflux_types::tool::{Tool, ToolContext};
use once_cell::sync::Lazy;
use redis::{AsyncCommands, Client};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::runtime::Handle;

static REDIS_REGISTRY: Lazy<Arc<RwLock<HashMap<String, Client>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct RedisTool;

impl Tool for RedisTool {
    fn id(&self) -> &'static str {
        "redis_query"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let connection_string = params["connection_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("connection_string is required"))?;
        
        let operation = params["operation"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("operation is required"))?;
        
        let key = params["key"].as_str().ok_or_else(|| anyhow::anyhow!("key is required"))?;

        let handle = Handle::current();
        
        handle.block_on(async {
            let client = self.get_or_create_client(connection_string).await?;
            let mut conn = client.get_multiplexed_tokio_connection().await?;

            match operation {
                "get" => {
                    let val: Option<String> = conn.get(key).await?;
                    Ok(json!({ "value": val }))
                }
                "set" => {
                    let val = params["value"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("value is required for set"))?;
                    let _: () = conn.set(key, val).await?;
                    Ok(json!({ "ok": true }))
                }
                "del" => {
                    let _: () = conn.del(key).await?;
                    Ok(json!({ "ok": true }))
                }
                _ => Err(anyhow::anyhow!("unsupported redis operation: {}", operation)),
            }
        })
    }
}

impl RedisTool {
    async fn get_or_create_client(&self, conn_str: &str) -> Result<Client> {
        {
            let registry = REDIS_REGISTRY.read().unwrap();
            if let Some(client) = registry.get(conn_str) {
                return Ok(client.clone());
            }
        }

        let client = Client::open(conn_str)?;

        let mut registry = REDIS_REGISTRY.write().unwrap();
        registry.insert(conn_str.to_string(), client.clone());
        
        Ok(client)
    }
}
