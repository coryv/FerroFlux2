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

    let edge_count = edge_query.iter().count();
    if edge_count > 0 && topology.adjacency.is_empty() {
        needs_rebuild = true;
    }

    if removed_edges.read().count() > 0 {
        needs_rebuild = true;
    }

    if needs_rebuild {
        tracing::debug!(edge_count = %edge_count, "Graph topology changed or uninitialized, rebuilding adjacency cache");
        topology.adjacency.clear();
        topology.inbound_connections.clear();

        for edge in edge_query.iter() {
            tracing::debug!(
                source = ?edge.source,
                target = ?edge.target,
                source_handle = ?edge.source_handle,
                "Cache rebuild: adding edge"
            );
            topology
                .adjacency
                .entry(edge.source)
                .or_default()
                .push((edge.source_handle.clone(), edge.target, edge.target_handle.clone()));

            if let Some(ref th) = edge.target_handle {
                topology
                    .inbound_connections
                    .entry(edge.target)
                    .or_default()
                    .insert(th.clone());
            }
        }
    }
}

/// System: Transport Worker (The Circulatory System)
///
/// **Role**: Moves Data Tickets from `Outbox` queues to connected `Inbox` queues.
#[tracing::instrument(skip(
    inbox_query,
    outbox_query,
    node_query,
    topology,
    work_done,
    bus,
    trace_query
))]
pub fn transport_worker(
    mut inbox_query: Query<&mut Inbox>,
    mut outbox_query: Query<&mut Outbox>,
    node_query: Query<(Entity, &NodeConfig)>, // Need to map Entity -> UUID
    topology: Res<GraphTopology>,
    mut work_done: ResMut<WorkDone>,
    bus: Res<SystemEventBus>,
    mut trace_query: Query<(
        &mut crate::components::observability::TraceNode,
        &crate::components::observability::Trace,
    )>,
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

            tracing::trace!(source = ?source, items = outbox.queue.len(), "Transport: outbox has items");

            // 3. Broadcast Tickets (Filtering by Port)
            let items: Vec<(Option<String>, SecureTicket)> = outbox.queue.drain(..).collect();

            for (port, ticket) in items {
                for (source_handle, target_entity, target_handle) in targets {
                    tracing::trace!(
                        port = ?port,
                        source_handle = ?source_handle,
                        target = ?target_entity,
                        "Transport: checking edge"
                    );
                    
                    // Match if:
                    // 1. Port matches source_handle (both Some and equal, OR both None)
                    // 2. Outbox port is None (generic broadcast)
                    let is_match = match (&port, &source_handle) {
                        (Some(p), Some(sh)) => p == sh,
                        (None, _) => true, // Generic broadcast from outbox
                        _ => false,
                    };

                    if is_match && let Ok(mut inbox) = inbox_query.get_mut(*target_entity) {
                        inbox.queue.push_back((target_handle.clone(), ticket.clone()));
                        tracing::debug!(source = ?source, target = ?target_entity, port = ?port, target_handle = ?target_handle, "Moved ticket");

                        let target_uuid = node_map.get(target_entity).cloned().unwrap_or_default();

                        tracing::trace!(
                            source = ?source,
                            target = ?target_entity,
                            trace_id = ?ticket.metadata.get("trace_id"),
                            "Moved ticket metadata check"
                        );

                        // Update Trace Entity if trace_id exists in ticket
                        if let Some(trace_id_str) = ticket.metadata.get("trace_id")
                            && let Ok(trace_uuid) = uuid::Uuid::parse_str(trace_id_str)
                        {
                            for (mut trace_node, trace) in trace_query.iter_mut() {
                                if trace.0 == trace_uuid {
                                    trace_node.0 = target_uuid;
                                }
                            }
                        }

                        // Signal Visualizer
                        let _ = bus.0.send(SystemEvent::EdgeTraversal {
                            source_id: node_map.get(source).cloned().unwrap_or_default(),
                            target_id: target_uuid,
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        });

                        work_done.0 = true;
                    }
                }
            }
        }
    }
}
