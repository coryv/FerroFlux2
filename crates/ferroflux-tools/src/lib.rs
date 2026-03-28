pub mod primitives;

use ferroflux_types::tool::ToolRegistry;
use ferroflux_types::Plugin;
use bevy_ecs::prelude::*;

/// Plugin for registering all core primitive tools.
pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, world: &mut World, _schedule: &mut Schedule) {
        if let Some(mut registry) = world.get_resource_mut::<ToolRegistry>() {
            register_core_tools(&mut registry);
        }
    }
}

/// Helper to register all core primitive tools into the given registry.
pub fn register_core_tools(registry: &mut ToolRegistry) {
    use primitives::*;
    registry.register(HttpClientTool);
    registry.register(PaginateTool);
    registry.register(JsonQueryTool);
    registry.register(EmitTool);
    registry.register(LogicTool);
    registry.register(LogTool);
    registry.register(SleepTool);
    registry.register(SetVarTool);
    registry.register(GetVarTool);
    registry.register(MathTool);
    registry.register(RhaiTool::default());
    registry.register(TraceTool);
    registry.register(StatsTool);
    registry.register(VerifySignatureTool);
}
