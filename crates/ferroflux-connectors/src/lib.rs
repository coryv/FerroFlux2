pub mod components;
pub mod nodes;
pub mod systems;

use crate::nodes::SseTriggerNodeFactory;
use bevy_ecs::prelude::*;
use ferroflux_types::registry::NodeRegistry;
use ferroflux_types::resources::SseTriggerRegistry;
use ferroflux_types::Plugin;

/// Plugin for registering all connector-related systems and components.
pub struct ConnectorsPlugin;

impl Plugin for ConnectorsPlugin {
    fn build(&self, world: &mut World, schedule: &mut Schedule) {
        // Register Resources
        if !world.contains_resource::<SseTriggerRegistry>() {
            world.insert_resource(SseTriggerRegistry::default());
        }

        // Register Nodes
        if let Some(mut registry) = world.get_resource_mut::<NodeRegistry>() {
            registry.register("SseTrigger", Box::new(SseTriggerNodeFactory));
        }

        // Register Systems
        schedule.add_systems((
            systems::rss_worker,
            systems::xml_worker,
            systems::ftp_worker,
            systems::ssh_worker,
            systems::sse_trigger_system,
        ));
    }
}
