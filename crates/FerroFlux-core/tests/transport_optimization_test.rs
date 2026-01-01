use bevy_ecs::prelude::*;
use ferroflux_core::components::{Edge, Inbox, NodeConfig, Outbox, WorkDone};
use ferroflux_core::resources::GraphTopology;
use ferroflux_core::store::SecureTicket;
use ferroflux_core::systems::transport::{transport_worker, update_graph_topology};
use std::collections::VecDeque;

#[test]
fn test_transport_optimization_dynamic_edge() {
    let mut world = World::new();

    // 1. Resources
    world.insert_resource(GraphTopology::default());
    world.insert_resource(WorkDone::default());
    let (tx, _) = tokio::sync::broadcast::channel(10);
    world.insert_resource(ferroflux_core::api::events::SystemEventBus(tx));

    // 2. Schedule
    let mut schedule = Schedule::default();
    schedule.add_systems((update_graph_topology, transport_worker).chain());

    // 3. Spawn Nodes
    let _source_id = uuid::Uuid::new_v4();
    let target_id = uuid::Uuid::new_v4();

    let source = world
        .spawn((
            NodeConfig {
                id: uuid::Uuid::new_v4(),
                name: "Node A".to_string(),
                node_type: "Generic".to_string(),
                workflow_id: None,
                tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
            },
            Outbox {
                queue: VecDeque::new(),
            },
            Inbox {
                queue: VecDeque::new(),
            },
        ))
        .id();

    let target = world
        .spawn((
            NodeConfig {
                id: target_id,
                name: "Target Node".to_string(),
                node_type: "Target".to_string(),
                workflow_id: None,
                tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
            },
            Outbox {
                queue: VecDeque::new(),
            },
            Inbox {
                queue: VecDeque::new(),
            },
        ))
        .id();

    // 4. Create dummy ticket
    let ticket = SecureTicket {
        id: uuid::Uuid::new_v4(),
        metadata: std::collections::HashMap::new(),
    };

    // 5. Push to Source Outbox
    world
        .get_mut::<Outbox>(source)
        .unwrap()
        .queue
        .push_back((Some("Exec".to_string()), ticket.clone()));

    // 6. Run Schedule WITHOUT Edge -> Should receive nothing
    schedule.run(&mut world);

    let target_inbox = world.get::<Inbox>(target).unwrap();
    assert!(
        target_inbox.queue.is_empty(),
        "Target should be empty before edge exists"
    );

    // 7. Add Edge dynamically
    world.spawn(Edge {
        source,
        target,
        source_handle: Some("Exec".to_string()),
        target_handle: Some("Exec".to_string()),
    });

    // 8. Run Schedule WITH Edge -> Topology should update, Ticket should move
    // We need to push ticket again because transport might have drained it?
    // Wait, transport `drain`s even if no targets in adjacency?
    // Let's check logic:
    // `for (source, targets) in &topology.adjacency`
    // If source not in adjacency, we don't query it. So we didn't drain!
    // Correct optimization!

    schedule.run(&mut world);

    let target_inbox = world.get::<Inbox>(target).unwrap();
    assert_eq!(
        target_inbox.queue.len(),
        1,
        "Target should receive ticket after edge addition"
    );
}
