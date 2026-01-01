use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortMetadata {
    pub name: String,
    pub data_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: String,
    pub name: String,
    pub category: String,
    pub platform: Option<String>,
    pub description: Option<String>,
    pub inputs: Vec<PortMetadata>,
    pub outputs: Vec<PortMetadata>,
    pub settings: Vec<Value>, // Using Value for schema to avoid circular deps for now
}

/// Trait for creating node entities from JSON configuration.
///
/// Implementing this allows a node type to be registered in the `NodeRegistry`.
pub trait NodeFactory: Send + Sync {
    /// Spawns the node's specific components onto the given entity.
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()>;

    /// Serializes the node's configuration from the ECS entity.
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value>;

    /// Returns metadata about the node for UI/docs.
    fn metadata(&self) -> NodeMetadata;
}
