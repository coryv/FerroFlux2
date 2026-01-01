use crate::nodes::definition::{NodeDefinition, PlatformDefinition};
use crate::traits::node_factory::NodeFactory;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default, Clone)]
pub struct DefinitionRegistry {
    pub definitions: HashMap<String, NodeDefinition>,
    pub platforms: HashMap<String, PlatformDefinition>,
}

impl DefinitionRegistry {
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.platforms.clear();
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

impl DefinitionRegistry {
    /// Recursively loads all .yaml node definitions from a directory.
    pub fn load_from_dir(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.load_from_dir(&path)?;
                } else if path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                {
                    self.load_file(&path)?;
                }
            }
        }
        Ok(())
    }

    fn load_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;

        // Try parsing as Platform first (simple structure)
        if let Ok(plat) = serde_yaml::from_str::<PlatformDefinition>(&content) {
            // Check if it's actually a platform (has config, no execution)
            // But NodeDefinition also has meta? Node doesn't have `config` field at top level usually?
            // Node has `execution`. Platform does not.
            // If it has `execution`, treat as Node.
            if content.contains("execution:") {
                let def: NodeDefinition = serde_yaml::from_str(&content)?;
                println!("DEBUG: Loading YAML Node: {}", def.meta.id);
                self.definitions.insert(def.meta.id.clone(), def);
                return Ok(());
            } else {
                println!("DEBUG: Loading YAML Platform: {}", plat.meta.id);
                self.platforms.insert(plat.meta.id.clone(), plat);
                return Ok(());
            }
        }

        // If strict parsing fails, try NodeDefinition specifically
        if let Ok(def) = serde_yaml::from_str::<NodeDefinition>(&content) {
            println!("DEBUG: Loading YAML Node: {}", def.meta.id);
            self.definitions.insert(def.meta.id.clone(), def);
            return Ok(());
        }

        Ok(())
    }
}
