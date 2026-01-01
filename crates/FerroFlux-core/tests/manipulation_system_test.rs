use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::components::core::{Inbox, NodeConfig, Outbox};
use ferroflux_core::components::manipulation::{
    AggregateConfig, BatchState, SplitConfig, TransformConfig,
};
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::manipulation::{aggregator_worker, splitter_worker, transform_worker};
use serde_json::json;
use tokio::sync::broadcast;
use uuid::Uuid;

// Helper to setup world
fn setup_world() -> World {
    let mut world = World::new();

    // Resources
    world.insert_resource(BlobStore::default());
    let (tx, _) = broadcast::channel(10);
    world.insert_resource(SystemEventBus(tx));

    world
}

#[test]
fn test_splitter_logic() {
    let mut world = setup_world();
    let mut schedule = Schedule::default();
    schedule.add_systems(splitter_worker);

    let store = world.resource::<BlobStore>().clone();

    // 1. Create Split Node
    let node_id = Uuid::new_v4();
    let config = SplitConfig {
        path: Some("items".to_string()),
    };

    let input_payload = json!({
        "items": [
            {"id": 1, "val": "A"},
            {"id": 2, "val": "B"}
        ]
    });
    let input_bytes = serde_json::to_vec(&input_payload).unwrap();
    let ticket = store.check_in(&input_bytes).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    world.spawn((
        NodeConfig {
            id: node_id,
            name: "Splitter".to_string(),
            node_type: "Split".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
        },
        config,
        inbox,
        Outbox::default(),
    ));

    // Run
    schedule.run(&mut world);

    // Verify Outbox
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    assert_eq!(outbox.queue.len(), 2);

    // Verify Content
    let (_, first_ticket) = &outbox.queue[0];
    let first_bytes = store.claim(first_ticket).unwrap();
    let first_val: serde_json::Value = serde_json::from_slice(&first_bytes).unwrap();
    assert_eq!(first_val["val"], "A");
}

#[test]
fn test_aggregator_logic() {
    let mut world = setup_world();
    let mut schedule = Schedule::default();
    schedule.add_systems(aggregator_worker);

    let store = world.resource::<BlobStore>().clone();

    // 1. Create Aggregate Node (Batch Size 2)
    let node_id = Uuid::new_v4();
    let config = AggregateConfig {
        batch_size: 2,
        timeout_seconds: 10,
    };

    let mut inbox = Inbox::default();
    // Push 2 items
    for i in 1..=2 {
        let p = json!({"id": i});
        let b = serde_json::to_vec(&p).unwrap();
        let t = store.check_in(&b).unwrap();
        inbox.queue.push_back(t);
    }

    world.spawn((
        NodeConfig {
            id: node_id,
            name: "Aggregator".to_string(),
            node_type: "Aggregate".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
        },
        config,
        BatchState::default(),
        inbox,
        Outbox::default(),
    ));

    // Run
    schedule.run(&mut world);

    // Verify Outbox (Should have 1 aggregate ticket)
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    assert_eq!(outbox.queue.len(), 1);

    let (_, batch_ticket) = &outbox.queue[0];
    let batch_bytes = store.claim(batch_ticket).unwrap();
    let batch_val: serde_json::Value = serde_json::from_slice(&batch_bytes).unwrap();

    assert!(batch_val.is_array());
    assert_eq!(batch_val.as_array().unwrap().len(), 2);
    assert_eq!(batch_val[0]["id"], 1);
}

#[test]
fn test_transform_logic() {
    let mut world = setup_world();
    let mut schedule = Schedule::default();
    schedule.add_systems(transform_worker);

    let store = world.resource::<BlobStore>().clone();

    // 1. Create Transform Node
    let node_id = Uuid::new_v4();
    let config = TransformConfig {
        expression: "users[].name".to_string(),
        result_key: Some("names".to_string()),
    };

    let input_payload = json!({
        "users": [
            {"name": "Alice"},
            {"name": "Bob"}
        ]
    });
    let input_bytes = serde_json::to_vec(&input_payload).unwrap();
    let ticket = store.check_in(&input_bytes).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    world.spawn((
        NodeConfig {
            id: node_id,
            name: "Transformer".to_string(),
            node_type: "Transform".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
        },
        config,
        inbox,
        Outbox::default(),
    ));

    // Run
    schedule.run(&mut world);

    // Verify Outbox
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    assert_eq!(outbox.queue.len(), 1);

    let (_, out_ticket) = &outbox.queue[0];
    let out_bytes = store.claim(out_ticket).unwrap();
    let out_val: serde_json::Value = serde_json::from_slice(&out_bytes).unwrap();

    // Assert Enrichment
    assert!(out_val["names"].is_array());
    assert_eq!(out_val["names"][0], "Alice");
    assert_eq!(out_val["names"][1], "Bob");
    assert!(out_val["users"].is_array()); // Original data preserved
}
