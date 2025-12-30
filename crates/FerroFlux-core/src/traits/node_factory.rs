use bevy_ecs::prelude::*;
use serde_json::Value;

/// Trait for creating node entities from JSON configuration.
///
/// Implementing this allows a node type to be registered in the `NodeRegistry`.
pub trait NodeFactory: Send + Sync {
    /// Spawns the node's specific components onto the given entity.
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()>;

    /// Serializes the node's configuration from the ECS entity.
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value>;
}
