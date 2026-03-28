use async_channel::{Receiver, Sender};
use bevy_ecs::prelude::*;
pub use ferroflux_types::registry::{DefinitionRegistry, NodeRegistry};
pub use ferroflux_types::resources::{GlobalHttpClient, TokioRuntime};
use std::sync::Arc;
use tokio::sync::Semaphore;
pub mod templates;

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
    // Source -> [(SourcePort, TargetEntity)]
    pub adjacency: std::collections::HashMap<Entity, Vec<(Option<String>, Entity)>>,
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
