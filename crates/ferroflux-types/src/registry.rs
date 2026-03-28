use crate::node_factory::{NodeFactory, NodeMetadata};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct NodeRegistry {
    factories: HashMap<String, Box<dyn NodeFactory>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, node_type: &str, factory: Box<dyn NodeFactory>) {
        self.factories.insert(node_type.to_lowercase(), factory);
    }

    pub fn get(&self, node_type: &str) -> Option<&dyn NodeFactory> {
        self.factories
            .get(&node_type.to_lowercase())
            .map(|b| b.as_ref())
    }

    pub fn clear(&mut self) {
        self.factories.clear();
    }

    pub fn list_templates(&self) -> Vec<NodeMetadata> {
        self.factories.values().map(|f| f.metadata()).collect()
    }
}
