use crate::components::pipeline::{ExecutionContext, ReadyToExecute};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Represents a job ready for execution by an integration provider.
/// This decouples the preparation logic from the actual execution environment.
#[derive(Debug, Clone)]
pub struct ExecutionJob {
    pub entity: Entity,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub trace_id: String,
    pub context: ExecutionContext,
}

/// Trait for dispatching execution jobs.
/// Implementations can be local (channel-based) or remote (Redis, API, etc.).
pub trait ExecutionBackend: Send + Sync + 'static {
    fn dispatch(&self, job: ExecutionJob) -> anyhow::Result<()>;
}

/// Wrapper resource for the execution backend.
#[derive(Resource)]
pub struct BackendResource(pub Arc<dyn ExecutionBackend>);

/// Default local implementation that sends jobs to a channel to be processed
/// by the ECS loop in the same or next tick.
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

/// System to flush local execution jobs from the channel into ECS components.
/// This bridges the gap between the decoupled backend and the local runner.
pub fn flush_local_execution_jobs(mut commands: Commands, receiver: Res<LocalExecutionReceiver>) {
    while let Ok(job) = receiver.0.try_recv() {
        commands.entity(job.entity).insert(ReadyToExecute {
            method: job.method,
            url: job.url,
            headers: job.headers,
            body: job.body,
            trace_id: job.trace_id,
            context: job.context,
        });
    }
}
