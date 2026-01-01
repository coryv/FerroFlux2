use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::components::manipulation::ExpressionConfig;
use ferroflux_core::components::{Inbox, NodeConfig, Outbox};
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::manipulation::expression_worker;
use serde_json::json;

#[test]
fn test_expression_evaluation() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // 1. Setup Resources
    let (tx, _) = tokio::sync::broadcast::channel(10);
    world.insert_resource(SystemEventBus(tx));
    let store = BlobStore::new();
    world.insert_resource(store.clone());

    // 2. Setup System
    schedule.add_systems(expression_worker);

    // 3. Setup Node Entity
    let node_id = uuid::Uuid::new_v4();
    let config = ExpressionConfig {
        expression: "(x + y) / 2".to_string(),
        result_key: "result".to_string(),
    };

    let node_config = NodeConfig {
        id: node_id,
        name: "Test Calc".to_string(),
        node_type: "Expression".to_string(),
        workflow_id: None,
        tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
    };

    let mut inbox = Inbox::default();

    // 4. Input: { x: 10, y: 20 }
    let payload = json!({ "x": 10.0, "y": 20.0 });
    let bytes = serde_json::to_vec(&payload).unwrap();
    let ticket = store.check_in(&bytes).unwrap();
    inbox.queue.push_back(ticket);

    let entity = world
        .spawn((config, node_config, inbox, Outbox::default()))
        .id();

    // 5. Run System
    schedule.run(&mut world);

    // 6. Verify Output
    let outbox = world.get::<Outbox>(entity).unwrap();
    assert_eq!(outbox.queue.len(), 1);

    let (_port, result_ticket) = outbox.queue.front().unwrap();
    let result_bytes = store.claim(result_ticket).unwrap();
    let result_json: serde_json::Value = serde_json::from_slice(&result_bytes).unwrap();

    // Expected: 15.0
    // "result": 15.0
    assert_eq!(result_json.get("result").unwrap().as_f64().unwrap(), 15.0);
    assert_eq!(result_json.get("x").unwrap().as_f64().unwrap(), 10.0);
}

#[test]
fn test_expression_functions() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    let (tx, _) = tokio::sync::broadcast::channel(10);
    world.insert_resource(SystemEventBus(tx));
    let store = BlobStore::new();
    world.insert_resource(store.clone());

    schedule.add_systems(expression_worker);

    let config = ExpressionConfig {
        expression: "sqrt(val)".to_string(),
        result_key: "root".to_string(),
    };

    let node_config = NodeConfig {
        id: uuid::Uuid::new_v4(),
        name: "Doubler".to_string(),
        node_type: "Expression".to_string(),
        workflow_id: None,
        tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
    };

    let mut inbox = Inbox::default();

    // Input: { val: 16 }
    let payload = json!({ "val": 16.0 });
    let bytes = serde_json::to_vec(&payload).unwrap();
    let ticket = store.check_in(&bytes).unwrap();
    inbox.queue.push_back(ticket);

    let entity = world
        .spawn((config, node_config, inbox, Outbox::default()))
        .id();

    schedule.run(&mut world);

    let outbox = world.get::<Outbox>(entity).unwrap();
    let (_port, result_ticket) = outbox.queue.front().unwrap();
    let result_bytes = store.claim(result_ticket).unwrap();
    let result_json: serde_json::Value = serde_json::from_slice(&result_bytes).unwrap();

    assert_eq!(result_json.get("root").unwrap().as_f64().unwrap(), 4.0);
}
