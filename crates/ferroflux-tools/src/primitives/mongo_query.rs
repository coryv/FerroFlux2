use anyhow::Result;
use ferroflux_types::tool::{Tool, ToolContext};
use futures::StreamExt;
use mongodb::{Client, options::ClientOptions, bson::Document};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::runtime::Handle;

static MONGO_REGISTRY: Lazy<Arc<RwLock<HashMap<String, Client>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct MongoTool;

impl Tool for MongoTool {
    fn id(&self) -> &'static str {
        "mongo_query"
    }

    fn run(&self, _context: &mut ToolContext, params: Value) -> Result<Value> {
        let connection_string = params["connection_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("connection_string is required"))?;
        
        let db_name = params["database"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("database is required"))?;
        
        let collection_name = params["collection"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("collection is required"))?;

        let operation = params["operation"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("operation is required"))?;

        let handle = Handle::current();
        
        handle.block_on(async {
            let client = self.get_or_create_client(connection_string).await?;
            let db = client.database(db_name);
            let coll = db.collection::<Document>(collection_name);

            match operation {
                "insert" => {
                    let doc_val = params["document"]
                        .as_object()
                        .ok_or_else(|| anyhow::anyhow!("document is required for insert"))?;
                    let doc: Document = serde_json::from_value(json!(doc_val))?;
                    let res = coll.insert_one(doc).await?;
                    Ok(json!({ "inserted_id": format!("{:?}", res.inserted_id) }))
                }
                "find" => {
                    let filter_val = params["filter"]
                        .as_object()
                        .cloned()
                        .unwrap_or_default();
                    let filter: Document = serde_json::from_value(json!(filter_val))?;
                    let mut cursor = coll.find(filter).await?;
                    let mut results = Vec::new();
                    while let Some(res) = cursor.next().await {
                        results.push(serde_json::to_value(res?)?);
                    }
                    Ok(json!({ "documents": results }))
                }
                "update" => {
                    let filter_val = params["filter"]
                        .as_object()
                        .ok_or_else(|| anyhow::anyhow!("filter is required for update"))?;
                    let update_val = params["update"]
                        .as_object()
                        .ok_or_else(|| anyhow::anyhow!("update is required for update"))?;
                    let filter: Document = serde_json::from_value(json!(filter_val))?;
                    let update: Document = serde_json::from_value(json!(update_val))?;
                    let res = coll.update_many(filter, update).await?;
                    Ok(json!({ "matched_count": res.matched_count, "modified_count": res.modified_count }))
                }
                "delete" => {
                    let filter_val = params["filter"]
                        .as_object()
                        .ok_or_else(|| anyhow::anyhow!("filter is required for delete"))?;
                    let filter: Document = serde_json::from_value(json!(filter_val))?;
                    let res = coll.delete_many(filter).await?;
                    Ok(json!({ "deleted_count": res.deleted_count }))
                }
                _ => Err(anyhow::anyhow!("unsupported mongo operation: {}", operation)),
            }
        })
    }
}

impl MongoTool {
    async fn get_or_create_client(&self, conn_str: &str) -> Result<Client> {
        {
            let registry = MONGO_REGISTRY.read().unwrap();
            if let Some(client) = registry.get(conn_str) {
                return Ok(client.clone());
            }
        }

        let options = ClientOptions::parse(conn_str).await?;
        let client = Client::with_options(options)?;

        let mut registry = MONGO_REGISTRY.write().unwrap();
        registry.insert(conn_str.to_string(), client.clone());
        
        Ok(client)
    }
}
