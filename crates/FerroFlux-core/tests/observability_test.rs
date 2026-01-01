use bevy_ecs::prelude::*;
use chrono::{Duration, Utc};
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::components::core::{Inbox, NodeConfig, Outbox};
use ferroflux_core::components::observability::*;
use ferroflux_core::resources::{GraphTopology, WorkDone};
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::janitor::janitor_worker;
use ferroflux_core::systems::transport::transport_worker;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_trace_propagation_via_transport() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Resources
    world.insert_resource(BlobStore::default());
    world.insert_resource(WorkDone::default());
    world.insert_resource(GraphTopology::default());
    let (tx, _) = tokio::sync::broadcast::channel(100);
    world.insert_resource(SystemEventBus(tx));

    // Nodes
    let node_a_id = Uuid::new_v4();
    let node_b_id = Uuid::new_v4();

    let node_a = world
        .spawn((
            NodeConfig {
                id: node_a_id,
                name: "Node A".to_string(),
                node_type: "test".to_string(),
                workflow_id: None,
                tenant_id: None,
            },
            Inbox::default(),
            Outbox::default(),
        ))
        .id();

    let node_b = world
        .spawn((
            NodeConfig {
                id: node_b_id,
                name: "Node B".to_string(),
                node_type: "test".to_string(),
                workflow_id: None,
                tenant_id: None,
            },
            Inbox::default(),
            Outbox::default(),
        ))
        .id();

    // Topology
    let mut topo = world.resource_mut::<GraphTopology>();
    topo.adjacency.insert(node_a, vec![(None, node_b)]);

    // Trace Entity
    let trace_id = Uuid::new_v4();
    world.spawn((
        Trace(trace_id),
        TraceNode(node_a_id),
        TraceStart(Utc::now()),
    ));

    // Push ticket to Node A outbox
    let mut metadata = HashMap::new();
    metadata.insert("trace_id".to_string(), trace_id.to_string());

    let ticket = world
        .resource::<BlobStore>()
        .check_in_with_metadata(b"{}", metadata)
        .unwrap();
    world
        .get_mut::<Outbox>(node_a)
        .unwrap()
        .queue
        .push_back((None, ticket));

    // Run transport
    schedule.add_systems(transport_worker);
    schedule.run(&mut world);

    // Verify TraceNode updated to Node B
    let mut trace_query = world.query::<&TraceNode>();
    let trace_node = trace_query.get_single(&world).unwrap();
    assert_eq!(trace_node.0, node_b_id);

    // Verify Node B inbox has the ticket
    let inbox_b = world.get::<Inbox>(node_b).unwrap();
    assert_eq!(inbox_b.queue.len(), 1);
}

#[test]
fn test_janitor_trace_pruning() {
    let mut world = World::new();
    let (tx, _) = tokio::sync::broadcast::channel(100);
    world.insert_resource(SystemEventBus(tx));
    world.insert_resource(BlobStore::default());
    world.insert_resource(ferroflux_core::systems::janitor::JanitorTimer::default());

    // Spawn fresh trace
    world.spawn((Trace(Uuid::new_v4()), TraceStart(Utc::now())));

    // Spawn old trace (2 hours ago)
    world.spawn((
        Trace(Uuid::new_v4()),
        TraceStart(Utc::now() - Duration::hours(2)),
    ));

    // Run janitor (after 11 seconds to trigger timer)
    let mut timer = world.resource_mut::<ferroflux_core::systems::janitor::JanitorTimer>();
    timer.0 = std::time::Instant::now() - std::time::Duration::from_secs(11);

    let mut schedule = Schedule::default();
    schedule.add_systems(janitor_worker);
    schedule.run(&mut world);

    // Verify only 1 trace remains
    let mut trace_query = world.query::<&Trace>();
    assert_eq!(trace_query.iter(&world).count(), 1);
}
