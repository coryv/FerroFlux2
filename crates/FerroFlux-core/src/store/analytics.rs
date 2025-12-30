use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct NoopStore;

#[async_trait]
impl AnalyticsBackend for NoopStore {
    async fn ingest_batch(&self, _events: Vec<AnalyticsEvent>) -> anyhow::Result<()> {
        Ok(())
    }
    async fn get_node_performance(
        &self,
        _tenant_id: &str,
        _node_id: &str,
    ) -> anyhow::Result<Vec<PerformanceMetric>> {
        Ok(vec![])
    }
    async fn get_recent_executions(
        &self,
        _tenant_id: &str,
        _limit: i64,
        _offset: i64,
    ) -> anyhow::Result<Vec<AnalyticsEvent>> {
        Ok(vec![])
    }
    async fn get_execution_events(
        &self,
        _tenant_id: &str,
        _trace_id: &str,
    ) -> anyhow::Result<Vec<AnalyticsEvent>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tenant_id: String,
    pub node_id: String,
    pub workflow_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub duration_ms: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub node_id: String,
    pub avg_duration_ms: f64,
    pub total_runs: i64,
    pub error_rate: f64,
}

#[async_trait]
pub trait AnalyticsBackend: Send + Sync {
    /// Ingests a batch of events.
    async fn ingest_batch(&self, events: Vec<AnalyticsEvent>) -> anyhow::Result<()>;

    /// Retrieves performance metrics for a specific node (or all nodes if node_id is empty).
    async fn get_node_performance(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> anyhow::Result<Vec<PerformanceMetric>>;

    /// Retrieves a list of recent execution events.
    async fn get_recent_executions(
        &self,
        tenant_id: &str,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<AnalyticsEvent>>;

    /// Retrieves all events for a specific execution trace.
    async fn get_execution_events(
        &self,
        tenant_id: &str,
        trace_id: &str,
    ) -> anyhow::Result<Vec<AnalyticsEvent>>;
}
