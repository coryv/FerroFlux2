use crate::api::handlers;
use crate::api::{ApiCommand, ApiReceiver};
use bevy_ecs::prelude::*;

/// System: API Command Consumer
///
/// **Role**: Bridges the Async World (Axum Web Server) and the Sync World (Bevy ECS).
///
/// **Mechanism**:
/// - The Web Server sends `ApiCommand` enums into a generic `async_channel`.
/// - This system drains that channel every tick.
/// - It performs "Exclusive World Access" operations like Spawning Entities (`LoadGraph`),
///   which cannot be done easily from async handlers.
#[tracing::instrument(skip(world))]
pub fn api_command_worker(world: &mut World) {
    if !world.contains_resource::<ApiReceiver>() {
        return;
    }

    let receiver = world.resource::<ApiReceiver>().0.clone();

    while let Ok(cmd) = receiver.try_recv() {
        let result = match cmd {
            ApiCommand::LoadGraph(tenant, yaml) => {
                handlers::graph::handle_load_graph(world, tenant, yaml)
            }
            ApiCommand::TriggerNode(tenant, uuid, payload) => {
                handlers::trigger::handle_trigger_node(world, tenant, uuid, payload)
            }
            ApiCommand::TriggerWorkflow(tenant, workflow_id, payload) => {
                handlers::trigger::handle_trigger_workflow(world, tenant, workflow_id, payload)
            }
            ApiCommand::PinNode(tenant, node_id, ticket_id) => {
                handlers::pin::handle_pin_node(world, tenant, node_id, ticket_id)
            }
            ApiCommand::ReloadDefinitions => handlers::registry::handle_reload_definitions(world),
        };

        if let Err(e) = result {
            tracing::error!(error = %e, "API command failed");
        }
    }
}
