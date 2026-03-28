pub mod components;
pub mod systems;

use bevy_ecs::prelude::*;
use ferroflux_types::Plugin;
use systems::WasmRuntime;

/// Plugin for registering WASM-based compute systems and resources.
pub struct WasmComputePlugin;

impl Plugin for WasmComputePlugin {
    fn build(&self, world: &mut World, schedule: &mut Schedule) {
        // Register Resources
        if !world.contains_resource::<WasmRuntime>() {
            world.insert_resource(WasmRuntime::default());
        }

        // Register Systems
        schedule.add_systems(systems::wasm_worker);
    }
}
