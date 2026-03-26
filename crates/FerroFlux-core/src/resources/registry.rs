use crate::traits::node_factory::NodeFactory;
use bevy_ecs::prelude::*;
use ferroflux_integration::DefinitionRegistry as IntegrationRegistry;
use std::collections::HashMap;

/// Bevy `Resource` wrapper around `ferroflux_integration::DefinitionRegistry`.
///
/// All loading, parsing, and validation live in the `ferroflux-integration` crate.
/// This wrapper exists solely to satisfy the Bevy ECS resource system.
#[derive(Resource, Default, Clone)]
pub struct DefinitionRegistry(pub IntegrationRegistry);

impl std::ops::Deref for DefinitionRegistry {
    type Target = IntegrationRegistry;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DefinitionRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

    pub fn list_templates(&self) -> Vec<crate::traits::node_factory::NodeMetadata> {
        self.factories.values().map(|f| f.metadata()).collect()
    }
}
