use crate::store::analytics::{AnalyticsBackend, AnalyticsEvent};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use tracing::error;

pub struct AnalyticsBatcher {
    tx: mpsc::UnboundedSender<AnalyticsEvent>,
    backend: Arc<dyn AnalyticsBackend>,
}

impl AnalyticsBatcher {
    pub fn new(backend: Arc<dyn AnalyticsBackend>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<AnalyticsEvent>();
        let backend_clone = backend.clone();

        tokio::spawn(async move {
            let mut buffer = Vec::with_capacity(1000);
            let mut interval = time::interval(Duration::from_secs(2));

            // Interval setup: first tick is immediate, so we consume it.
            interval.tick().await;

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        buffer.push(event);
                        if buffer.len() >= 1000 {
                            let batch = std::mem::take(&mut buffer);
                            if let Err(e) = backend_clone.ingest_batch(batch).await {
                                error!("Failed to flush analytics batch (size limit): {}", e);
                            }
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            let batch = std::mem::take(&mut buffer);
                            if let Err(e) = backend_clone.ingest_batch(batch).await {
                                error!("Failed to flush analytics batch (time limit): {}", e);
                            }
                        }
                    }
                }
            }
        });

        Self { tx, backend }
    }

    pub fn backend(&self) -> &Arc<dyn AnalyticsBackend> {
        &self.backend
    }

    pub fn track(&self, event: AnalyticsEvent) {
        if let Err(e) = self.tx.send(event) {
            error!("Failed to send analytics event to batcher: {}", e);
        }
    }
}
