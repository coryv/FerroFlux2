use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::{Edge, Inbox, Outbox, core::NodeConfig};
use crate::resources::{GraphTopology, WorkDone};
use crate::store::SecureTicket;
use bevy_ecs::prelude::*;

/// System: Update Graph Topology
///
/// **Role**: maintains the `GraphTopology` resource, which is an optimized adjacency cache.
///
/// **Trigger**: Runs only when `Edge` components are Added or Changed (and ideally Removed).
///
/// **Logic**:
/// - If any edge changes, it clears the cache and rebuilds the entire adjacency list.
/// - This turns an O(E) lookup into O(1) for the `transport_worker`.
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip(topology, changed_edges, edge_query, removed_edges))]
pub fn update_graph_topology(
    mut topology: ResMut<GraphTopology>,
    changed_edges: Query<Entity, Or<(Changed<Edge>, Added<Edge>)>>,
    edge_query: Query<&Edge>,
    // TODO: Handle removed edges if necessary (RemovedComponents<Edge>)
    // For now, if ANY edge changes/adds, we rebuild entire valid map.
    // Ideally we would also check RemovedComponents to trigger rebuild.
    mut removed_edges: RemovedComponents<Edge>,
) {
    let mut needs_rebuild = false;

    if !changed_edges.is_empty() {
        needs_rebuild = true;
    }

    // Check for removals (this iterator drains events, so we must check it even if others are empty)
    if removed_edges.read().count() > 0 {
        needs_rebuild = true;
    }

    if needs_rebuild {
        tracing::info!("Graph topology changed, rebuilding adjacency cache");
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
///
/// **Mental Model**:
/// - Acts as the "Physics Engine" of data flow. It doesn't care *what* the data is, only where it goes.
/// - It uses the `GraphTopology` cache to avoid iterating all edges every frame.
///
/// **Algorithm**:
/// 1. Build a temporary mapping of `Entity -> UUID` (to support distributed systems later).
/// 2. Iterate only *Active Sources* found in `topology.adjacency`.
/// 3. Drain the `Outbox` of each source.
/// 4. Clone and Push tickets to the `Inbox` of every connected Target.
/// 5. Emit `EdgeTraversal` events for UI visualization.
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

    if !topology.adjacency.is_empty() {
        println!(
            "DEBUG: Transport Worker: Topology Size: {}",
            topology.adjacency.len()
        );
    }

    tracing::debug!(
        "Transport Worker: Topology Size: {}",
        topology.adjacency.len()
    );

    // 2. Iterate Sources with Active Connections (from cache)
    for (source, targets) in &topology.adjacency {
        if let Ok(mut outbox) = outbox_query.get_mut(*source) {
            if outbox.queue.is_empty() {
                continue;
            }

            // 3. Broadcast Tickets (Filtering by Port)
            // Outbox queue is now (Option<String>, SecureTicket)
            let items: Vec<(Option<String>, SecureTicket)> = outbox.queue.drain(..).collect();

            for (port, ticket) in items {
                println!(
                    "DEBUG: Processing ticket from {:?} on port {:?}",
                    source, port
                );
                for (edge_handle, target_entity) in targets {
                    println!(
                        "DEBUG: Checking edge {:?} -> {:?}",
                        edge_handle, target_entity
                    );
                    tracing::debug!(
                        "Transport: Checking edge {:?} -> {:?}",
                        edge_handle,
                        target_entity
                    );
                    // Exact match on port name (handle).
                    // If outbox says "Success", only edges from "Success" fire.
                    if edge_handle == &port {
                        match inbox_query.get_mut(*target_entity) {
                            Ok(mut inbox) => {
                                inbox.queue.push_back(ticket.clone());
                                tracing::debug!(source = ?source, target = ?target_entity, port = ?port, "Moved ticket");
                                println!(
                                    "DEBUG: Moved ticket from {:?} to {:?}",
                                    source, target_entity
                                );

                                // Signal Visualizer
                                let _ = bus.0.send(SystemEvent::EdgeTraversal {
                                    source_id: node_map.get(source).cloned().unwrap_or_default(),
                                    target_id: node_map
                                        .get(target_entity)
                                        .cloned()
                                        .unwrap_or_default(),
                                    timestamp: chrono::Utc::now().timestamp_millis(),
                                });

                                work_done.0 = true;
                            }
                            Err(e) => {
                                println!(
                                    "DEBUG: Failed to get Inbox for {:?}: {:?}",
                                    target_entity, e
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
