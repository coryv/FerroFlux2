use anyhow::{Context, Result};
use ferroflux_types::tool::{Tool, ToolContext};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sqlx::{AnyConnection, AnyPool, any::AnyPoolOptions, Column, Row};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::runtime::Handle;

static POOL_REGISTRY: Lazy<Arc<RwLock<HashMap<String, AnyPool>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

// In-memory registry for active transactions, keyed by flow trace_id.
// Note: This is an ephemeral solution for local execution.
static TX_REGISTRY: Lazy<Arc<tokio::sync::Mutex<HashMap<String, AnyConnection>>>> =
    Lazy::new(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())));

// TODO: Implement proper numeric type mapping (currently String-only)
// This will require mapping sqlx::any::AnyTypeInfo to appropriate serde_json::Value variants.

pub struct SqlTool;

impl Tool for SqlTool {
    fn id(&self) -> &'static str {
        "sql_query"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let connection_string = params["connection_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("connection_string is required"))?;
        
        let operation = params["operation"].as_str().unwrap_or("execute");
        let trace_id = &context.trace_id;

        let handle = Handle::current();
        
        handle.block_on(async {
            match operation {
                "begin" => {
                    let pool = self.get_or_create_pool(connection_string).await?;
                    let mut conn = pool.acquire().await?.detach();
                    sqlx::query("BEGIN").execute(&mut conn).await?;
                    
                    let mut registry = TX_REGISTRY.lock().await;
                    registry.insert(trace_id.clone(), conn);
                    Ok(json!({ "status": "transaction_started" }))
                }
                "commit" => {
                    let mut registry = TX_REGISTRY.lock().await;
                    if let Some(mut conn) = registry.remove(trace_id) {
                        sqlx::query("COMMIT").execute(&mut conn).await?;
                        Ok(json!({ "status": "transaction_committed" }))
                    } else {
                        Err(anyhow::anyhow!("No active transaction for trace_id: {}", trace_id))
                    }
                }
                "rollback" => {
                    let mut registry = TX_REGISTRY.lock().await;
                    if let Some(mut conn) = registry.remove(trace_id) {
                        sqlx::query("ROLLBACK").execute(&mut conn).await?;
                        Ok(json!({ "status": "transaction_rolled_back" }))
                    } else {
                        Err(anyhow::anyhow!("No active transaction for trace_id: {}", trace_id))
                    }
                }
                "execute" => {
                    let query = params["query"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("query is required"))?;
                    
                    let bind_params = params["bind"].as_array();

                    // Check for active transaction
                    let mut registry = TX_REGISTRY.lock().await;
                    if let Some(conn) = registry.get_mut(trace_id) {
                        self.execute_with_executor(conn, query, bind_params).await
                    } else {
                        let pool = self.get_or_create_pool(connection_string).await?;
                        self.execute_with_executor(&pool, query, bind_params).await
                    }
                }
                _ => Err(anyhow::anyhow!("Unsupported SQL operation: {}", operation)),
            }
        })
    }
}

impl SqlTool {
    async fn execute_with_executor<'q, E>(&self, executor: E, query: &'q str, bind_params: Option<&Vec<Value>>) -> Result<Value>
    where E: sqlx::Executor<'q, Database = sqlx::Any> {
        let mut sql_query = sqlx::query(query);
        
        if let Some(args) = bind_params {
            for arg in args {
                match arg {
                    Value::String(s) => sql_query = sql_query.bind(s.clone()),
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            sql_query = sql_query.bind(i);
                        } else if let Some(f) = n.as_f64() {
                            sql_query = sql_query.bind(f);
                        }
                    }
                    Value::Bool(b) => sql_query = sql_query.bind(*b),
                    _ => sql_query = sql_query.bind(arg.to_string()),
                }
            }
        }

        let rows = sql_query.fetch_all(executor).await?;

        let result_rows: Vec<Value> = rows.into_iter().map(|row| {
            let mut map = serde_json::Map::new();
            for col in row.columns() {
                let name = col.name();
                // TODO: Apply proper numeric type mapping here (refer to result mapping task)
                let val: Value = row.try_get::<String, _>(name)
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                map.insert(name.to_string(), val);
            }
            Value::Object(map)
        }).collect();

        Ok(json!({ "rows": result_rows }))
    }

    async fn get_or_create_pool(&self, conn_str: &str) -> Result<AnyPool> {
        {
            let registry = POOL_REGISTRY.read().unwrap();
            if let Some(pool) = registry.get(conn_str) {
                return Ok(pool.clone());
            }
        }

        sqlx::any::install_default_drivers();

        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(conn_str)
            .await
            .context(format!("Failed to connect to {}", conn_str))?;

        let mut registry = POOL_REGISTRY.write().unwrap();
        registry.insert(conn_str.to_string(), pool.clone());
        
        Ok(pool)
    }
}
