use super::{AnalyticsBackend, AnalyticsEvent, PerformanceMetric};
use anyhow::Result;
use async_trait::async_trait;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Row, Serialize, Deserialize)]
struct ClickHouseEvent {
    id: Uuid,
    timestamp: i64, // ClickHouse DateTime64(3) often expects milliseconds if using i64 or specific format.
    // Actually clickhouse crate handles DateTime<Utc> usually if feature enabled, otherwise i64 is safer for epoch ms.
    // Let's use i64 for epoch ms to be safe with `toYYYYMM` partition relying on types.
    // Or just let serde handle it?
    // Let's use i64 ms.
    tenant_id: String,
    node_id: String,
    workflow_id: String,
    event_type: String,
    payload: String, // Store JSON as String
    duration_ms: u32,
    status: String,
}

impl From<AnalyticsEvent> for ClickHouseEvent {
    fn from(e: AnalyticsEvent) -> Self {
        Self {
            id: e.id,
            timestamp: e.timestamp.timestamp_millis(),
            tenant_id: e.tenant_id,
            node_id: e.node_id,
            workflow_id: e.workflow_id,
            event_type: e.event_type,
            payload: e.payload.to_string(),
            duration_ms: e.duration_ms as u32,
            status: e.status,
        }
    }
}

pub struct ClickHouseStore {
    client: Client,
}

impl ClickHouseStore {
    pub fn new(url: &str) -> Self {
        let client = Client::default().with_url(url).with_database("default"); // Could be configurable
        Self { client }
    }

    // Schema init must be done separately or via specialized query
    pub async fn init_schema(&self) -> Result<()> {
        let schema_sql =
            std::fs::read_to_string("assets/sql/analytics_clickhouse.sql").or_else(|_| {
                Ok::<String, anyhow::Error>(
                    include_str!("../../../assets/sql/analytics_clickhouse.sql").to_string(),
                )
            })?;

        // Split by ; if multiple queries? Clickhouse crate usually expects one query per call.
        // The file has CREATE TABLE.
        self.client.query(&schema_sql).execute().await?;
        Ok(())
    }
}

#[async_trait]
impl AnalyticsBackend for ClickHouseStore {
    async fn ingest_batch(&self, events: Vec<AnalyticsEvent>) -> Result<()> {
        let mut insert = self.client.insert("analytics_events")?;
        for event in events {
            let row: ClickHouseEvent = event.into();
            insert.write(&row).await?;
        }
        insert.end().await?;
        Ok(())
    }

    async fn get_recent_executions(
        &self,
        _tenant_id: &str,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<AnalyticsEvent>> {
        // Not implemented yet
        Ok(vec![])
    }

    async fn get_execution_events(
        &self,
        _tenant_id: &str,
        _trace_id: &str,
    ) -> Result<Vec<AnalyticsEvent>> {
        // Not implemented yet
        Ok(vec![])
    }

    async fn get_node_performance(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<Vec<PerformanceMetric>> {
        // Query logic for ClickHouse
        let mut query_str = String::from(
            r#"
            SELECT 
                node_id, 
                avg(duration_ms) as avg_duration, 
                count(*) as total_runs,
                countIf(status = 'error') / count(*) as error_rate
            FROM analytics_events
            WHERE tenant_id = ?
            "#,
        );

        if !node_id.is_empty() {
            query_str.push_str(" AND node_id = ?");
        }

        query_str.push_str(" GROUP BY node_id");

        let mut query = self.client.query(&query_str).bind(tenant_id);

        if !node_id.is_empty() {
            query = query.bind(node_id);
        }

        let cursor = query.fetch_all::<ClickHouseMetric>().await?;
        Ok(cursor.into_iter().map(|m| m.into()).collect())
    }
}

// Helper for fetching metrics
#[derive(Row, Deserialize)]
struct ClickHouseMetric {
    node_id: String,
    avg_duration: f64,
    total_runs: u64,
    error_rate: f64,
}

impl From<ClickHouseMetric> for PerformanceMetric {
    fn from(m: ClickHouseMetric) -> Self {
        Self {
            node_id: m.node_id,
            avg_duration_ms: m.avg_duration,
            total_runs: m.total_runs as i64,
            error_rate: m.error_rate,
        }
    }
}
