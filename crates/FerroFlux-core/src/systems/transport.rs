use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::{Edge, Inbox, Outbox, core::NodeConfig};
use crate::resources::{GraphTopology, WorkDone};
use crate::store::SecureTicket;
use bevy_ecs::prelude::*;

/// System: Update Graph Topology
///
/// **Role**: maintains the `GraphTopology` resource, which is an optimized adjacency cache.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip(topology, changed_edges, edge_query, removed_edges))]
pub fn update_graph_topology(
    mut topology: ResMut<GraphTopology>,
    changed_edges: Query<Entity, Or<(Changed<Edge>, Added<Edge>)>>,
    edge_query: Query<&Edge>,
    mut removed_edges: RemovedComponents<Edge>,
) {
    let mut needs_rebuild = false;

    // Rebuild if any edge changed/added or if the cache is currently empty but world has edges.
    if !changed_edges.is_empty() {
        needs_rebuild = true;
    }

    if removed_edges.read().count() > 0 {
        needs_rebuild = true;
    }

    // Special case: if topology is empty but there are edges, we must rebuild (handle startup/tests)
    if topology.adjacency.is_empty() && !edge_query.is_empty() {
        needs_rebuild = true;
    }

    if needs_rebuild {
        tracing::debug!("Graph topology changed or uninitialized, rebuilding adjacency cache");
        topology.adjacency.clear();
        for edge in edge_query.iter() {
            topology
                .adjacency
                .entry(edge.source)
                .or_default()
                .push((edge.source_handle.clone(), edge.target));
        }
    }
}

/// System: Transport Worker (The Circulatory System)
///
/// **Role**: Moves Data Tickets from `Outbox` queues to connected `Inbox` queues.
#[tracing::instrument(skip(inbox_query, outbox_query, node_query, topology, work_done, bus))]
pub fn transport_worker(
    mut inbox_query: Query<&mut Inbox>,
    mut outbox_query: Query<&mut Outbox>,
    node_query: Query<(Entity, &NodeConfig)>, // Need to map Entity -> UUID
    topology: Res<GraphTopology>,
    mut work_done: ResMut<WorkDone>,
    bus: Res<SystemEventBus>,
) {
    // 1. Build Entity -> UUID Map (Optimization: Move to resource if slow)
    let node_map: std::collections::HashMap<Entity, uuid::Uuid> =
        node_query.iter().map(|(e, c)| (e, c.id)).collect();

    // 2. Iterate Sources with Active Connections (from cache)
    for (source, targets) in &topology.adjacency {
        if let Ok(mut outbox) = outbox_query.get_mut(*source) {
            if outbox.queue.is_empty() {
                continue;
            }

            // 3. Broadcast Tickets (Filtering by Port)
            let items: Vec<(Option<String>, SecureTicket)> = outbox.queue.drain(..).collect();

            for (port, ticket) in items {
                for (edge_handle, target_entity) in targets {
                    // Exact match on port name (handle).
                    // If outbox says "Success", only edges from "Success" fire.
                    if edge_handle == &port {
                        if let Ok(mut inbox) = inbox_query.get_mut(*target_entity) {
                            inbox.queue.push_back(ticket.clone());
                            tracing::debug!(source = ?source, target = ?target_entity, port = ?port, "Moved ticket");

                            // Signal Visualizer
                            let _ = bus.0.send(SystemEvent::EdgeTraversal {
                                source_id: node_map.get(source).cloned().unwrap_or_default(),
                                target_id: node_map.get(target_entity).cloned().unwrap_or_default(),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            });

                            work_done.0 = true;
                        }
                    }
                }
            }
        }
    }
}
