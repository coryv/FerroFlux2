use super::{AnalyticsBackend, AnalyticsEvent, PerformanceMetric};
use anyhow::Result;
use async_trait::async_trait;
use duckdb::Connection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct DuckDbStore {
    // DuckDB connection is not Sync, so we wrap in Mutex
    conn: Arc<Mutex<Connection>>,
}

impl DuckDbStore {
    pub async fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Initialize schema
        // Note: For now we embed the SQL or read it. Since we are in src, we can use include_str!
        // if we put the sql in src, but it is in assets.
        // We will read it relative to CWD.
        let schema_sql =
            std::fs::read_to_string("assets/sql/analytics_duckdb.sql").or_else(|_| {
                Ok::<String, anyhow::Error>(
                    include_str!("../../../assets/sql/analytics_duckdb.sql").to_string(),
                )
            })?;

        conn.execute_batch(&schema_sql)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl AnalyticsBackend for DuckDbStore {
    async fn get_recent_executions(
        &self,
        tenant_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AnalyticsEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                id, timestamp, tenant_id, node_id, workflow_id, event_type, payload, duration_ms, status
            FROM analytics_events
            WHERE tenant_id = ?
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#,
        )?;

        let rows = stmt.query_map(duckdb::params![tenant_id, limit, offset], |row| {
            let payload_str: String = row.get(6)?;
            let payload: serde_json::Value =
                serde_json::from_str(&payload_str).unwrap_or(serde_json::json!({}));

            Ok(AnalyticsEvent {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: row.get(1)?,
                tenant_id: row.get(2)?,
                node_id: row.get(3)?,
                workflow_id: row.get(4)?,
                event_type: row.get(5)?,
                payload,
                duration_ms: row.get(7)?,
                status: row.get(8)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    async fn ingest_batch(&self, events: Vec<AnalyticsEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let conn = self.conn.lock().unwrap();
        let mut appender = conn.appender("analytics_events")?;

        for event in events {
            appender.append_row(duckdb::params![
                event.id.to_string(),
                event.timestamp, // DuckDb handles Chrono DateTime
                event.tenant_id,
                event.node_id,
                event.workflow_id,
                event.event_type,
                event.payload.to_string(), // Serialize to JSON string for DuckDB storage
                event.duration_ms,
                event.status
            ])?;
        }

        // Appender autoflushes on drop usually, or we can explicit flush if available?
        // duckdb-rs appender flushes on drop.
        Ok(())
    }

    async fn get_node_performance(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Vec<PerformanceMetric>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                node_id, 
                AVG(duration_ms) as avg_duration, 
                COUNT(*) as total_runs,
                SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END)::DOUBLE / COUNT(*)::DOUBLE as error_rate
            FROM analytics_events
            WHERE tenant_id = ?
            AND (? = '' OR node_id = ?)
            GROUP BY node_id
            "#,
        )?;

        let _node_id_param = node_id.to_string();
        let rows = stmt.query_map(duckdb::params![tenant_id, node_id, node_id], |row| {
            Ok(PerformanceMetric {
                node_id: row.get(0)?,
                avg_duration_ms: row.get(1)?,
                total_runs: row.get(2)?,
                error_rate: row.get(3)?,
            })
        })?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row?);
        }

        Ok(metrics)
    }
    async fn get_execution_events(
        &self,
        tenant_id: &str,
        trace_id: &str,
    ) -> Result<Vec<AnalyticsEvent>> {
        let conn = self.conn.lock().unwrap();
        // DuckDB JSON filtering: json_extract(payload, '$.trace_id')
        // We also check "payload" column.
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                id, timestamp, tenant_id, node_id, workflow_id, event_type, payload, duration_ms, status
            FROM analytics_events
            WHERE tenant_id = ?
            AND json_extract_string(payload, '$.trace_id') = ?
            ORDER BY timestamp ASC
            "#,
        )?;

        let rows = stmt.query_map(duckdb::params![tenant_id, trace_id], |row| {
            let payload_str: String = row.get(6)?;
            let payload: serde_json::Value =
                serde_json::from_str(&payload_str).unwrap_or(serde_json::json!({}));

            Ok(AnalyticsEvent {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: row.get(1)?,
                tenant_id: row.get(2)?,
                node_id: row.get(3)?,
                workflow_id: row.get(4)?,
                event_type: row.get(5)?,
                payload,
                duration_ms: row.get(7)?,
                status: row.get(8)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }
}
