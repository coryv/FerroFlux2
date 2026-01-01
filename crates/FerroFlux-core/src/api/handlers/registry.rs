use crate::api::PlatformPath;
use crate::nodes::register_core_nodes;
use crate::nodes::yaml_factory::YamlNodeFactory;
use crate::resources::registry::{DefinitionRegistry, NodeRegistry};
use bevy_ecs::prelude::*;

pub fn handle_reload_definitions(world: &mut World) -> anyhow::Result<()> {
    tracing::info!("Processing ReloadDefinitions command");

    let path_opt = world.get_resource::<PlatformPath>().cloned();

    if let Some(path_res) = path_opt {
        let path = path_res.0;
        tracing::info!(path = ?path, "Reloading definitions from path");

        // 1. Refresh DefinitionRegistry
        if let Some(mut registry) = world.get_resource_mut::<DefinitionRegistry>() {
            registry.clear();
            registry.load_from_dir(&path)?;
        }

        // 2. Re-bridge to NodeRegistry
        let def_registry_clone = world.get_resource::<DefinitionRegistry>().cloned();

        if let Some(defs) = def_registry_clone {
            if let Some(mut registry) = world.get_resource_mut::<NodeRegistry>() {
                registry.clear();
                register_core_nodes(&mut registry);
                for (id, def) in &defs.definitions {
                    registry.register(id, Box::new(YamlNodeFactory::new(def.clone())));
                }
                tracing::info!(count = defs.definitions.len(), "Node factories reloaded");
            }
        }
        Ok(())
    } else {
        Err(anyhow::anyhow!("PlatformPath resource not found"))
    }
}
