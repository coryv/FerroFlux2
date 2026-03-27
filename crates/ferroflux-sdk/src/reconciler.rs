use anyhow::Result;
use bevy_ecs::prelude::*;
use ferroflux_core::components::core::{Edge, Inbox, NodeConfig, Outbox};
use flow_canvas::model::{GraphState, NodeData};
use std::collections::{HashMap, HashSet};

/// Reconciles the visual `GraphState` with the running ECS `World`.
///
/// This function ensures that:
/// 1. New nodes in the Graph are spawned in the World.
/// 2. Removed nodes in the Graph are despawned from the World.
/// 3. Edges are rebuilt (simplified strategy for V1).
/// 4. Existing nodes have their config updated (if supported).
///
/// This preserves the *runtime state* (e.g., mailboxes, memory) of nodes that persist.
pub fn reconcile_graph<T: NodeData>(world: &mut World, graph: &GraphState<T>) -> Result<()> {
    // 1. Index Existing Nodes in World
    let mut existing_entities = HashMap::new();
    let mut query = world.query_filtered::<(Entity, &NodeConfig), With<NodeConfig>>();
    for (entity, config) in query.iter(world) {
        existing_entities.insert(config.id, entity);
    }

    // 2. Identify Changes
    let graph_node_ids: HashSet<_> = graph.nodes.values().map(|n| n.uuid).collect();
    let world_node_ids: HashSet<_> = existing_entities.keys().cloned().collect();

    let to_spawn: Vec<_> = graph_node_ids.difference(&world_node_ids).collect();
    let to_despawn: Vec<_> = world_node_ids.difference(&graph_node_ids).collect();

    // 3. Despawn Removed Nodes
    for uuid in to_despawn {
        if let Some(entity) = existing_entities.get(uuid) {
            world.despawn(*entity);
            // Look for associated resources/components if complex cleanup is needed?
            // Bevy handles component cleanup on despawn.
            tracing::debug!("Reconciler: Despawned node {}", uuid);
        }
    }

    // 4. Update Existing Nodes (Settings/Config)
    // For V1, we just update the NodeConfig component.
    // Ideally, we'd check if specific fields changed to avoid spurious change detection.
    for node in graph.nodes.values() {
        if let Some(entity) = existing_entities.get(&node.uuid)
            && let Some(mut config) = world.get_mut::<NodeConfig>(*entity) {
                // Update Name / Type if changed?
                // Type change usually implies full respawn, but for now we assume same type.
                config.name = format!("{:?}", node.id);
                // We DON'T change execution state here.
            }
    }

    // 5. Spawn New Nodes
    let mut canvas_to_entity = HashMap::new(); // Map Graph UUID -> Entity

    // Re-populate map with existing valid entities
    for (uuid, entity) in &existing_entities {
        if graph_node_ids.contains(uuid) {
            canvas_to_entity.insert(*uuid, *entity);
        }
    }

    for uuid in to_spawn {
        // Find the node data in the graph (reverse lookup needed or iterate)
        if let Some(node) = graph.nodes.values().find(|n| n.uuid == *uuid) {
            let entity = world
                .spawn((
                    NodeConfig {
                        id: node.uuid,
                        name: format!("{:?}", node.id),
                        node_type: node.data.node_type(),
                        workflow_id: "global".to_string(),
                        tenant_id: ferroflux_iam::TenantId::from("default"),
                    },
                    Inbox::default(),
                    Outbox::default(),
                    ferroflux_core::components::pipeline::PipelineNode::new(
                        node.data.node_type(),
                        std::collections::HashMap::new(),
                    ),
                ))
                .id();

            canvas_to_entity.insert(*uuid, entity);
            tracing::debug!("Reconciler: Spawned node {}", uuid);
        }
    }

    // 6. Rebuild Edges
    // Edge reconciliation is harder (directed cyclic graph).
    // V1 Strategy: Clear ALL edges and re-spawn them.
    // This is safe because Edges are usually stateless data carriers in ECS *definition*,
    // though `Outbox/Inbox` hold the *active* data.
    // The `Edge` component itself is just topology.

    // Despawn all existing edges
    let mut edge_query = world.query_filtered::<Entity, With<Edge>>();
    let edges: Vec<Entity> = edge_query.iter(world).collect();
    for e in edges {
        world.despawn(e);
    }

    // Spawn current edges
    for (_, conn) in &graph.connections {
        let from_node_uuid = graph
            .ports
            .get(conn.from)
            .and_then(|p| graph.nodes.get(p.node).map(|n| n.uuid));
        let to_node_uuid = graph
            .ports
            .get(conn.to)
            .and_then(|p| graph.nodes.get(p.node).map(|n| n.uuid));

        if let (Some(from_uuid), Some(to_uuid)) = (from_node_uuid, to_node_uuid)
            && let (Some(&src_entity), Some(&target_entity)) = (
                canvas_to_entity.get(&from_uuid),
                canvas_to_entity.get(&to_uuid),
            ) {
                // In future, map port names correctly
                world.spawn(Edge {
                    source: src_entity,
                    target: target_entity,
                    source_handle: Some("Exec".to_string()),
                    target_handle: Some("Exec".to_string()),
                });
            }
    }

    Ok(())
}
