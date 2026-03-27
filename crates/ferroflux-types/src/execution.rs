use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// A job encapsulating everything needed to make an HTTP API call.
///
/// `ExecutionJob` decouples the preparation of an integration request from
/// the actual HTTP dispatch, enabling remote or distributed execution.
#[derive(Debug, Clone)]
pub struct ExecutionJob {
    pub entity: Entity,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub trace_id: String,
}

/// Trait for dispatching integration execution jobs.
///
/// Implementations can be local (in-process channel) or remote (Redis, SQS, etc.).
pub trait ExecutionBackend: Send + Sync + 'static {
    fn dispatch(&self, job: ExecutionJob) -> anyhow::Result<()>;
}

/// Bevy resource wrapping the active `ExecutionBackend`.
#[derive(Resource)]
pub struct BackendResource(pub Arc<dyn ExecutionBackend>);

/// Default local implementation — sends jobs through an async channel to be
/// processed by the ECS loop in the same or next tick.
#[derive(Clone)]
pub struct LocalExecutionBackend {
    sender: async_channel::Sender<ExecutionJob>,
}

impl LocalExecutionBackend {
    pub fn new(sender: async_channel::Sender<ExecutionJob>) -> Self {
        Self { sender }
    }
}

impl ExecutionBackend for LocalExecutionBackend {
    fn dispatch(&self, job: ExecutionJob) -> anyhow::Result<()> {
        self.sender
            .send_blocking(job)
            .map_err(|e| anyhow::anyhow!("Failed to dispatch job: {}", e))
    }
}

/// Resource to receive jobs locally and inject them back into ECS.
#[derive(Resource)]
pub struct LocalExecutionReceiver(pub async_channel::Receiver<ExecutionJob>);
