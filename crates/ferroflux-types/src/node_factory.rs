use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Metadata about a single input or output port on a node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortMetadata {
    pub name: String,
    pub data_type: String,
}

/// Metadata describing a node type, used for the node library UI and documentation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: String,
    pub name: String,
    pub category: String,
    pub platform: Option<String>,
    pub description: Option<String>,
    pub inputs: Vec<PortMetadata>,
    pub outputs: Vec<PortMetadata>,
    /// JSON Schema for settings — stored as `Value` to avoid circular deps.
    pub settings: Vec<Value>,
}

/// Trait for creating and serializing node entities in the ECS world.
///
/// Implementing this trait allows a node type to be registered in the
/// `NodeRegistry` and instantiated from JSON configuration at runtime.
pub trait NodeFactory: Send + Sync {
    /// Spawns the node's specific ECS components onto the given entity.
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()>;

    /// Serializes the node's current configuration from the ECS world.
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value>;

    /// Returns static metadata describing this node type.
    fn metadata(&self) -> NodeMetadata;

    /// Resolves the concrete port list for a specific configuration instance.
    ///
    /// The default implementation returns the static metadata ports. Override
    /// this for nodes with dynamic port counts (e.g. merge nodes).
    fn resolve_interface(&self, _config: &Value) -> (Vec<PortMetadata>, Vec<PortMetadata>) {
        let meta = self.metadata();
        (meta.inputs, meta.outputs)
    }
}
