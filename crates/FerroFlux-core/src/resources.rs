use async_channel::{Receiver, Sender};
use bevy_ecs::prelude::*;
pub use ferroflux_types::registry::NodeRegistry;
pub use ferroflux_types::resources::{GlobalHttpClient, TokioRuntime};

/// Bevy `Resource` wrapper around `ferroflux_integration::DefinitionRegistry`.
///
/// Lives here (in core) rather than in `ferroflux-types` so that the foundational
/// types crate does not depend on the domain-specific integration crate.
#[derive(Resource, Default, Clone)]
pub struct DefinitionRegistry(pub ferroflux_integration::DefinitionRegistry);

impl std::ops::Deref for DefinitionRegistry {
    type Target = ferroflux_integration::DefinitionRegistry;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DefinitionRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
use std::sync::Arc;
use tokio::sync::Semaphore;
pub mod templates;

#[derive(Resource, Clone, Default)]
pub struct WorkDone(pub bool);

#[derive(Resource, Clone, Default)]
pub struct GlobalMemory(pub std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>>);

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

pub type AdjacencyMap = std::collections::HashMap<Entity, Vec<(Option<String>, Entity, Option<String>)>>;

#[derive(Resource, Clone, Default)]
pub struct GraphTopology {
    // Source -> [(SourcePort, TargetEntity, TargetPort)]
    pub adjacency: AdjacencyMap,
    // Target -> Set of Target Port Names that have incoming edges
    pub inbound_connections: std::collections::HashMap<Entity, std::collections::HashSet<String>>,
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
