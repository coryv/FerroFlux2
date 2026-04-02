use crate::components::SseTriggerConfig;
use bevy_ecs::prelude::*;
use ferroflux_types::{NodeConfig, NodeFactory, NodeMetadata, Outbox};
use serde_json::Value;

pub struct SseTriggerNodeFactory;

impl NodeFactory for SseTriggerNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
        let c: SseTriggerConfig = serde_json::from_value(config.clone())?;

        // Extract metadata if present to ensure NodeConfig is consistent
        if let Some(mut node_config) = entity.get_mut::<NodeConfig>() {
            node_config.node_type = "SseTrigger".to_string();
        }

        entity.insert(c);
        entity.insert(Outbox::default());

        Ok(())
    }

    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<SseTriggerConfig>(entity)
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
    }

    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "SseTrigger".to_string(),
            name: "SSE Trigger".to_string(),
            category: "Connectors".to_string(),
            description: Some("Listen for Server-Sent Events (SSE)".to_string()),
            platform: None,
            inputs: vec![],
            outputs: vec![ferroflux_types::PortMetadata {
                name: "default".to_string(),
                data_type: "Any".to_string(),
            }],
            settings: vec![],
            discovery: None,
        }
    }
}
