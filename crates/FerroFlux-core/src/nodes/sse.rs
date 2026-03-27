use crate::traits::node_factory::{NodeFactory, NodeMetadata, PortMetadata};
use bevy_ecs::prelude::*;
use serde_json::Value;

pub struct SseTriggerNodeFactory;

impl NodeFactory for SseTriggerNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
        let sse_config: crate::components::connectors::SseTriggerConfig = serde_json::from_value(config.clone())?;
        entity.insert(sse_config);
        Ok(())
    }

    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<crate::components::connectors::SseTriggerConfig>(entity)
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "SseTrigger".to_string(),
            name: "SSE Trigger".to_string(),
            category: "Trigger".to_string(),
            platform: None,
            description: Some("Listen for Server-Sent Events from an external URL".to_string()),
            inputs: vec![],
            outputs: vec![PortMetadata {
                name: "default".to_string(),
                data_type: "Object".to_string(),
            }],
            settings: vec![],
        }
    }
}
