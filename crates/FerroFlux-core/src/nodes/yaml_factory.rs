use crate::components::pipeline::PipelineNode;
use crate::nodes::definition::NodeDefinition;
use crate::traits::node_factory::{NodeFactory, NodeMetadata, PortMetadata};
use anyhow::Result;
use bevy_ecs::prelude::*;
use serde_json::Value;

/// A Generic Node Factory that spawns entities based on a YAML definition.
pub struct YamlNodeFactory {
    pub definition: NodeDefinition,
}

impl YamlNodeFactory {
    pub fn new(definition: NodeDefinition) -> Self {
        Self { definition }
    }
}

impl NodeFactory for YamlNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> Result<()> {
        // Create the runtime component
        // Current config is merged with defaults in the pipeline system,
        // but here we just persist the instance config.
        let config_map: std::collections::HashMap<String, Value> =
            serde_json::from_value(config.clone()).unwrap_or_default();

        let node = PipelineNode::new(self.definition.meta.id.clone(), config_map);

        entity.insert(node);
        entity.insert(crate::components::Inbox::default());
        entity.insert(crate::components::Outbox::default());
        Ok(())
    }

    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<PipelineNode>(entity)
            .map(|n| serde_json::to_value(&n.config).unwrap_or(Value::Null))
    }

    fn metadata(&self) -> NodeMetadata {
        let meta = &self.definition.meta;
        let interface = &self.definition.interface;

        NodeMetadata {
            id: meta.id.clone(),
            name: meta.name.clone(),
            category: meta.category.clone(),
            platform: meta.platform.clone(),
            description: meta.description.clone(),
            inputs: interface
                .inputs
                .iter()
                .map(|p| PortMetadata {
                    name: p.name.clone(),
                    data_type: p.data_type.clone(),
                })
                .collect(),
            outputs: interface
                .outputs
                .iter()
                .map(|p| PortMetadata {
                    name: p.name.clone(),
                    data_type: p.data_type.clone(),
                })
                .collect(),
            settings: interface
                .settings
                .iter()
                .map(|s| serde_json::to_value(s).unwrap())
                .collect(),
        }
    }
}
