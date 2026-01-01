use crate::components::pipeline::PipelineNode;
use anyhow::{Context, Result};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

pub fn handle_simulate_node(
    world: &mut World,
    _tenant: ferroflux_iam::TenantId,
    node_id: uuid::Uuid,
    input_ticket: uuid::Uuid,
    trace_id: String,
    mock_config: HashMap<String, crate::components::shadow::MockConfig>,
) -> Result<()> {
    // 1. Resolve Node Definition and Config
    // This is tricky: Do we simulate a node *definition* or an *instance*?
    // Usually instance. We need its config settings.
    // However, existing nodes are entities.
    // To Simulate, we can clone an existing node's config.
    // BUT we don't have a direct "Map<Uuid, NodeConfig>" easily accessible without query.

    // Assumption: The user provides the node_id of a deployed node entity.
    // We need to find that entity to get its configuration.
    // But direct Entity lookup by UUID requires iteration or a lookup resource.
    // "ferroflux_iam::TenantId" implies multi-tenancy.

    // For MVP, let's scan components::core::NodeMap if it exists, or just query.
    // Actually, `crate::components::core::NodeMap` is `HashMap<Entity, Uuid>`.
    // We need `Uuid -> Entity`.

    // Let's iterate all PipelineNodes to find the matching config.
    // This is O(N) but acceptable for MVP simulation trigger.

    // DIFFERENTIATION: SimulateNode creates a NEW ephemeral entity to avoid state corruption.
    // So we *must* find the config of the source node.

    let source_entity = find_entity_by_uuid(world, node_id).context("Node not found")?;

    // Extract config
    let (definition_id, config) = {
        let node = world.entity(source_entity);
        let p_node = node.get::<PipelineNode>().context("Not a pipeline node")?;
        (p_node.definition_id.clone(), p_node.config.clone())
    };

    // 2. Spawn Ephemeral Mutation
    // "Shadow Entity"
    let shadow_node = PipelineNode {
        definition_id,
        config,
        execution_context: Default::default(),
    };

    // 3. Prepare Input
    // We need to fetch the ticket to ensure it exists?
    // Or just queue it. The pipeline system loads it.
    let ticket = crate::store::SecureTicket {
        id: input_ticket,
        metadata: {
            let mut m = HashMap::new();
            m.insert("trace_id".to_string(), trace_id);
            m.insert("shadow".to_string(), "true".to_string());
            m
        },
    };

    let mut inbox = crate::components::Inbox::default();
    inbox.queue.push_back(ticket);

    // 4. Spawn
    world.spawn((
        shadow_node,
        inbox,
        crate::components::Outbox::default(),
        crate::components::shadow::ShadowExecution {
            mocked_tools: mock_config,
        },
        // We might want a cleanup component or TTL so these don't pile up?
        // For now, rely on "Janitor"? Janitor cleans traces, not entities.
        // We should add `Ephemeral` component.
    ));

    tracing::info!(%node_id, "Spawned ephemeral shadow node");

    Ok(())
}

fn find_entity_by_uuid(world: &mut World, target: uuid::Uuid) -> Option<Entity> {
    let mut target_entity = None;
    let mut query = world.query::<(Entity, &crate::components::NodeConfig)>();
    for (e, conf) in query.iter(world) {
        if conf.id == target {
            target_entity = Some(e);
            break;
        }
    }
    target_entity
}
