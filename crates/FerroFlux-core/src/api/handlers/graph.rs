use crate::graph_loader::load_graph_from_str;
use bevy_ecs::prelude::*;
use ferroflux_iam::TenantId;

pub fn handle_load_graph(world: &mut World, tenant: TenantId, yaml: String) -> anyhow::Result<()> {
    tracing::info!("Processing LoadGraph command");
    load_graph_from_str(world, tenant, &yaml)
}
