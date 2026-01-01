use chrono::Utc;
use ferroflux_core::api::events::SystemEvent;
use ferroflux_core::store::analytics::{AnalyticsBackend, AnalyticsEvent};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use uuid::Uuid;

pub struct AnalyticsBatcher<B: AnalyticsBackend> {
    backend: Arc<B>,
    bus: broadcast::Sender<SystemEvent>,
    batch_size: usize,
    interval: Duration,
}

impl<B: AnalyticsBackend + 'static> AnalyticsBatcher<B> {
    pub fn new(
        backend: B,
        bus: broadcast::Sender<SystemEvent>,
        batch_size: usize,
        interval_secs: u64,
    ) -> Self {
        Self {
            backend: Arc::new(backend),
            bus,
            batch_size,
            interval: Duration::from_secs(interval_secs),
        }
    }

    pub async fn run(self) {
        let mut interval = time::interval(self.interval);
        let mut rx = self.bus.subscribe();
        let mut buffer = Vec::new();

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(event) => {
                            if let Some(analytics_event) = self.convert_event(event) {
                                buffer.push(analytics_event);
                                if buffer.len() >= self.batch_size {
                                    let batch = std::mem::take(&mut buffer);
                                    let _ = self.backend.ingest_batch(batch).await;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(missed = n, "Analytics batcher lagged");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        let batch = std::mem::take(&mut buffer);
                        let _ = self.backend.ingest_batch(batch).await;
                    }
                }
            }
        }
    }

    fn convert_event(&self, event: SystemEvent) -> Option<AnalyticsEvent> {
        match event {
            SystemEvent::NodeTelemetry {
                trace_id,
                node_id,
                node_type,
                execution_ms,
                success,
                details,
            } => {
                let mut payload = details;
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("trace_id".to_string(), serde_json::Value::String(trace_id));
                }
                Some(AnalyticsEvent {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    tenant_id: "default".to_string(),
                    node_id: node_id.to_string(),
                    workflow_id: "".to_string(),
                    event_type: node_type,
                    payload,
                    duration_ms: execution_ms,
                    status: if success {
                        "success".to_string()
                    } else {
                        "error".to_string()
                    },
                })
            }
            SystemEvent::NodeError {
                trace_id,
                node_id,
                error,
                ..
            } => Some(AnalyticsEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                tenant_id: "default".to_string(),
                node_id: node_id.to_string(),
                workflow_id: "".to_string(),
                event_type: "error".to_string(),
                payload: serde_json::json!({ "error": error, "trace_id": trace_id }),
                duration_ms: 0,
                status: "error".to_string(),
            }),
            _ => None,
        }
    }
}
