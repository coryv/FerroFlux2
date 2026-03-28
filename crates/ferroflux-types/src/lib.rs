pub mod blob;
pub mod components;
pub mod data_ref;
pub mod events;
pub mod execution;
pub mod node_factory;
pub mod registry;
pub mod resources;
pub mod shadow;
pub mod tenant;
pub mod tool;
pub mod trigger;
pub mod utils;

// Convenience re-exports
pub use blob::{BlobProvider, BlobSnapshot, BlobStore, SecureTicket};
pub use components::{Edge, EdgeLabel, Inbox, NodeConfig, Outbox, PinnedOutput};
pub use data_ref::{ActiveWorkflowState, DataRef};
pub use events::{SystemEvent, SystemEventBus};
pub use execution::{BackendResource, ExecutionBackend, ExecutionJob, LocalExecutionBackend};
pub use node_factory::{NodeFactory, NodeMetadata, PortMetadata};
pub use registry::{DefinitionRegistry, NodeRegistry};
pub use resources::{GlobalHttpClient, SseTriggerRegistry, TokioRuntime};
pub use shadow::{MockConfig, ShadowExecution};
pub use tenant::TenantId;
pub use tool::{Tool, ToolContext, ToolRegistry};
pub use trigger::{TriggerEvent, TriggerProvider, TriggerReceiver, TriggerSender};
pub use utils::merge_result;

/// A trait for modular extensions to the FerroFlux engine.
///
/// Plugins can register systems, resources, and nodes to the ECS world/schedule.
pub trait Plugin: Send + Sync {
    fn build(&self, world: &mut bevy_ecs::prelude::World, schedule: &mut bevy_ecs::prelude::Schedule);
}
