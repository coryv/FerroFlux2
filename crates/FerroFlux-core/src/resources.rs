use async_channel::{Receiver, Sender};
use bevy_ecs::prelude::*;
use std::sync::Arc;
use tokio::sync::Semaphore;
pub mod registry;
pub mod templates;

pub use registry::NodeRegistry;

#[derive(Resource, Clone, Debug)]
pub struct TokioRuntime(pub tokio::runtime::Handle);

#[derive(Resource, Clone)]
pub struct GlobalHttpClient {
    pub client: reqwest::Client,
}

impl Default for GlobalHttpClient {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap();
        Self { client }
    }
}

#[derive(Resource, Clone, Default)]
pub struct WorkDone(pub bool);

#[derive(Resource, Clone)]
pub struct AgentConcurrency(pub Arc<Semaphore>);

#[derive(Resource, Clone)]
pub struct AgentResultChannel {
    pub tx: Sender<(Entity, String, std::collections::HashMap<String, String>)>,
    pub rx: Receiver<(Entity, String, std::collections::HashMap<String, String>)>,
}

impl Default for AgentResultChannel {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self { tx, rx }
    }
}
#[derive(Resource, Clone, Default)]
pub struct NodeRouter(pub std::collections::HashMap<uuid::Uuid, Entity>);

#[derive(Resource, Clone)]
pub struct HttpResultChannel {
    pub tx: Sender<(Entity, String, std::collections::HashMap<String, String>)>,
    pub rx: Receiver<(Entity, String, std::collections::HashMap<String, String>)>,
}

impl Default for HttpResultChannel {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self { tx, rx }
    }
}

#[derive(Resource, Clone, Default)]
pub struct GraphTopology {
    // Source -> [Targets]
    pub adjacency: std::collections::HashMap<Entity, Vec<Entity>>,
}
#[derive(Resource, Clone)]
pub struct PipelineResultChannel {
    pub tx: Sender<(Entity, crate::components::pipeline::ExecutionResult)>,
    pub rx: Receiver<(Entity, crate::components::pipeline::ExecutionResult)>,
}

impl Default for PipelineResultChannel {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self { tx, rx }
    }
}
