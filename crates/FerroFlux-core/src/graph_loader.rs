use crate::components::{Edge, EdgeLabel, Inbox, NodeConfig, Outbox, SecretConfig};
use bevy_ecs::prelude::*;
use ferroflux_iam::TenantId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeBlueprint {
    pub source_id: String,
    pub target_id: String,
    pub label: Option<String>,
    pub source_handle: Option<String>,
    pub target_handle: Option<String>,
}

/// The structure of the YAML file.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowBlueprint {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    /// List of triggers (optional section for semantic grouping).
    pub triggers: Option<Vec<NodeBlueprint>>,
    /// List of nodes to spawn.
    pub nodes: Vec<NodeBlueprint>,
    /// List of connections between nodes.
    pub edges: Vec<EdgeBlueprint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeBlueprint {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<SecretConfig>,
}

fn get_stable_uuid(id_str: &str) -> Uuid {
    // Attempt to parse as UUID first
    if let Ok(u) = Uuid::parse_str(id_str) {
        u
    } else {
        // Fallback to stable namespaced v5 UUID
        let namespace = Uuid::NAMESPACE_DNS;
        Uuid::new_v5(&namespace, id_str.as_bytes())
    }
}

pub fn load_graph(world: &mut World, tenant: TenantId, path: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)?;
    load_graph_from_str(world, tenant, &content)
}

/// Parses a YAML workflow definition and spawns the corresponding ECS entities.
#[tracing::instrument(skip(world, tenant, yaml))]
pub fn load_graph_from_str(world: &mut World, tenant: TenantId, yaml: &str) -> anyhow::Result<()> {
    let blueprint: WorkflowBlueprint = serde_yaml::from_str(yaml)?;

    let mut uuid_map: HashMap<Uuid, Entity> = HashMap::new();

    let parsed_id: Option<String> = serde_yaml::from_str::<serde_json::Value>(yaml)
        .ok()
        .and_then(|v| v.get("id").and_then(|s| s.as_str()).map(|s| s.to_string()));
    let workflow_id_ref = parsed_id.clone();

    // 0. CLEANUP: Despawn existing entities for this workflow
    if let Some(wf_id) = &workflow_id_ref {
        let mut to_despawn = Vec::new();
        let mut query = world.query::<(Entity, &NodeConfig)>();
        for (e, conf) in query.iter(world) {
            if conf.workflow_id == *wf_id {
                to_despawn.push(e);
            }
        }

        // Also need to cleanup edges connected to these nodes?
        // Edges have source/target Entity IDs. If nodes are despawned, edges might dangle or be despawned automatically?
        // Bevy doesn't auto-despawn dependent entities unless parented. Mine are just components.
        // But `Edge` struct has `Entity` fields.
        // Actually, cleaner to rely on `Edge` component entities.
        // If I delete nodes, I should probably delete edges too.
        // Edges don't store workflow_id directly.
        // But if I find all edges where source OR target is in `to_despawn`, I should delete them.

        // Let's find edges first
        let mut edges_to_despawn = Vec::new();
        let mut edge_query = world.query::<(Entity, &Edge)>();
        for (e, edge) in edge_query.iter(world) {
            if to_despawn.contains(&edge.source) || to_despawn.contains(&edge.target) {
                edges_to_despawn.push(e);
            }
        }

        tracing::info!(node_count = to_despawn.len(), edge_count = edges_to_despawn.len(), workflow_id = %wf_id, "Cleaning up old graph entities");

        for e in edges_to_despawn {
            world.despawn(e);
        }
        for e in to_despawn {
            world.despawn(e);
        }
    }

    // 1. Spawn Nodes (including triggers)
    let mut all_nodes = blueprint.nodes;
    if let Some(mut triggers) = blueprint.triggers {
        all_nodes.append(&mut triggers);
    }

    for node_bp in all_nodes {
        let node_id = get_stable_uuid(&node_bp.id);
        let node_name = node_bp.name.clone();
        let node_type = node_bp.node_type.clone();

        let entity = world
            .spawn((
                NodeConfig {
                    id: node_id,
                    name: node_name.clone(),
                    node_type: node_type.clone(),
                    workflow_id: workflow_id_ref
                        .clone()
                        .unwrap_or_else(|| "global".to_string()),
                    tenant_id: tenant.clone(),
                },
                Inbox::default(),
                Outbox::default(),
            ))
            .id();

        // Use Registry to add variant-specific components
        world.resource_scope(
            |world, registry: Mut<ferroflux_types::registry::NodeRegistry>| {
                if let Some(factory) = registry.get(&node_type) {
                    let mut entity_mut = world.entity_mut(entity);
                    if let Err(e) = factory.build(&mut entity_mut, &node_bp.config) {
                        tracing::error!(node_name = %node_name, error = %e, "Error building node");
                    }
                } else {
                    tracing::warn!(node_type = %node_type, "No factory found for node type");
                }
            },
        );

        // Add Secret if present
        if let Some(secret) = node_bp.secret {
            world.entity_mut(entity).insert(secret);
        }

        uuid_map.insert(node_id, entity);
        tracing::info!(entity = ?entity, node_name = %node_name, node_type = %node_type, "Spawned Node");
    }

    // 2. Spawn Edges
    for edge_bp in blueprint.edges {
        let source_uuid = get_stable_uuid(&edge_bp.source_id);
        let target_uuid = get_stable_uuid(&edge_bp.target_id);

        let source = *uuid_map
            .get(&source_uuid)
            .ok_or_else(|| anyhow::anyhow!("Edge source ID not found: {}", edge_bp.source_id))?;
        let target = *uuid_map
            .get(&target_uuid)
            .ok_or_else(|| anyhow::anyhow!("Edge target ID not found: {}", edge_bp.target_id))?;

        let mut edge_cmds = world.spawn(Edge {
            source,
            target,
            source_handle: edge_bp.source_handle.clone(),
            target_handle: edge_bp.target_handle.clone(),
        });

        if let Some(label) = edge_bp.label {
            edge_cmds.insert(EdgeLabel(label));
        }
        tracing::info!(source = ?source, target = ?target, "Spawned Edge");
    }

    // 3. Populate NodeRouter (for O(1) Webhook lookups)
    if let Some(mut router) = world.get_resource_mut::<crate::resources::NodeRouter>() {
        for (id, entity) in &uuid_map {
            router.0.insert(*id, *entity);
        }
        tracing::info!(count = uuid_map.len(), "Populated NodeRouter");
    }

    Ok(())
}

// NOTE: Save Logic is omitted for now as prompt only requested "load_from_file" for verification,
// and "We can define a workflow in a YAML file... and the engine executes it".
// But "Save Logic" was in Step 3 plan. I will implement a basic save stub or full if needed.
// Step 3 says "Save Logic... Iterate all entities...". I'll implement it for completeness.

pub fn save_graph(world: &mut World, path: &str) -> anyhow::Result<()> {
    // Basic implementation for now, might need further refactoring for full registry support
    let mut nodes: Vec<NodeBlueprint> = Vec::new();
    let mut edges: Vec<EdgeBlueprint> = Vec::new();

    let node_entities: Vec<(Entity, NodeConfig)> = world
        .query::<(Entity, &NodeConfig)>()
        .iter(world)
        .map(|(e, c)| (e, c.clone()))
        .collect();

    for (e, node_config) in node_entities {
        let mut config_json = serde_json::json!({});
        world.resource_scope(
            |world, registry: Mut<ferroflux_types::registry::NodeRegistry>| {
                if let Some(factory) = registry.get(&node_config.node_type)
                    && let Some(c) = factory.serialize(world, e)
                {
                    config_json = c;
                }
            },
        );

        nodes.push(NodeBlueprint {
            id: node_config.id.to_string(),
            name: node_config.name,
            node_type: node_config.node_type,
            config: config_json,
            secret: world.get::<SecretConfig>(e).cloned(),
        });
    }

    // 2. Query Edges
    let mut edge_query = world.query::<(Entity, &Edge, Option<&EdgeLabel>)>();
    for (_, edge, label) in edge_query.iter(world) {
        // Resolve Source/Target Entity -> UUID using Config
        let get_uuid = |e: Entity| -> Option<Uuid> { world.get::<NodeConfig>(e).map(|c| c.id) };

        if let (Some(source_id), Some(target_id)) = (get_uuid(edge.source), get_uuid(edge.target)) {
            edges.push(EdgeBlueprint {
                source_id: source_id.to_string(),
                target_id: target_id.to_string(),
                label: label.map(|l| l.0.clone()),
                source_handle: edge.source_handle.clone(),
                target_handle: edge.target_handle.clone(),
            });
        }
    }

    let blueprint = WorkflowBlueprint {
        id: Some("exported_workflow".to_string()),
        name: "exported_workflow".to_string(),
        description: Some("Generated from ECS runtime".to_string()),
        triggers: None,
        nodes,
        edges,
    };
    let file = std::fs::File::create(path)?;
    serde_yaml::to_writer(file, &blueprint)?;

    Ok(())
}
