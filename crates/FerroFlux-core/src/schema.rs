use crate::components::{Edge, ExpectedOutput, NodeConfig, Requirements};
use bevy_ecs::prelude::*;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::{HashMap, HashSet};

/// Propagates requirements from sink nodes upstream to source nodes.
///
/// Algorithm:
/// 1. Build an adjacency graph of all entities with `NodeConfig` and `Edge` components.
/// 2. Perform a topological sort.
/// 3. Iterate through the nodes in *reverse* topological order (children before parents).
/// 4. For each node, calculate its `ExpectedOutput` as the union of its own `Requirements`
///    and the `ExpectedOutput` of all its downstream children.
#[tracing::instrument(skip(world))]
pub fn propagate_requirements(world: &mut World) {
    // 1. Build Graph
    let mut graph = DiGraph::<Entity, ()>::new();
    let mut entity_to_node = HashMap::new();

    // Add all nodes
    let mut node_query = world.query::<(Entity, &NodeConfig)>();
    for (entity, _) in node_query.iter(world) {
        let node_idx = graph.add_node(entity);
        entity_to_node.insert(entity, node_idx);
    }

    // Add all edges
    let mut edge_query = world.query::<&Edge>();
    for edge in edge_query.iter(world) {
        if let (Some(&src), Some(&target)) = (
            entity_to_node.get(&edge.source),
            entity_to_node.get(&edge.target),
        ) {
            graph.add_edge(src, target, ());
        }
    }

    // 2. Topological Sort
    // toposort returns nodes in order: Parent -> Child
    // We want Child -> Parent (Reverse Topo), so we reverse the result.
    let sorted_nodes = match toposort(&graph, None) {
        Ok(nodes) => nodes,
        Err(_) => {
            tracing::warn!("Cycle detected in graph! Schema propagation aborted.");
            return;
        }
    };

    // 3. Reverse Walk
    for node_idx in sorted_nodes.into_iter().rev() {
        let entity = *graph.node_weight(node_idx).unwrap();

        // Collect requirements from this node
        let local_reqs = if let Some(req) = world.get::<Requirements>(entity) {
            req.needed_fields.clone()
        } else {
            HashSet::new()
        };

        // Collect aggregated requirements from children (ExpectedOutput)
        let mut aggregated_reqs = local_reqs;

        // Find children in the graph
        let mut children = Vec::new();
        // neighbors_directed(Outgoing) gives us the children
        for child_idx in graph.neighbors_directed(node_idx, petgraph::Direction::Outgoing) {
            let child_entity = *graph.node_weight(child_idx).unwrap();
            children.push(child_entity);
        }

        for child_entity in children {
            if let Some(child_output) = world.get::<ExpectedOutput>(child_entity) {
                // Union child's expectations into our current set
                for field in &child_output.aggregated_schema {
                    aggregated_reqs.insert(field.clone());
                }
            }
        }

        // 4. Save to ExpectedOutput
        // We use get_mut_or_insert (if Bevy had it) or just ensure component exists.
        // Since we are iterating strictly, we can assume we want to attach this component.
        // However, we must be careful not to invalidate pointers if we were querying,
        // but here we are using direct entity access.

        let mut entity_mut = world.entity_mut(entity);

        if let Some(mut output) = entity_mut.get_mut::<ExpectedOutput>() {
            output.aggregated_schema = aggregated_reqs;
        } else {
            entity_mut.insert(ExpectedOutput {
                aggregated_schema: aggregated_reqs,
            });
        }
    }
}
