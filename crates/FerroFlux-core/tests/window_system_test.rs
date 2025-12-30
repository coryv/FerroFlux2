use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::components::manipulation::{WindowConfig, WindowOp, WindowState};
use ferroflux_core::components::{Inbox, NodeConfig, Outbox};
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::manipulation::window_worker;
use serde_json::json;
// use std::collections::VecDeque;

#[test]
fn test_window_rolling_mean() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // 1. Setup Resources
    let (tx, _) = tokio::sync::broadcast::channel(10);
    world.insert_resource(SystemEventBus(tx));
    let store = BlobStore::new();
    world.insert_resource(store.clone());

    // 2. Setup System
    schedule.add_systems(window_worker);

    // 3. Setup Node Entity
    let node_id = uuid::Uuid::new_v4();
    let config = WindowConfig {
        target_field: "value".to_string(),
        result_key: "rolling_mean".to_string(),
        operation: WindowOp::Mean,
        window_size: 3,
    };

    let node_config = NodeConfig {
        id: node_id,
        name: "Test Window".to_string(),
        node_type: "Window".to_string(),
        workflow_id: None,
        tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
    };

    let mut inbox = Inbox::default();

    // 4. Populate Inbox with sequence: 10, 20, 30, 40
    let inputs = vec![10.0, 20.0, 30.0, 40.0];
    for val in inputs {
        let payload = json!({ "value": val });
        let bytes = serde_json::to_vec(&payload).unwrap();
        let ticket = store.check_in(&bytes).unwrap();
        inbox.queue.push_back(ticket);
    }

    let entity = world
        .spawn((
            config,
            node_config,
            WindowState::default(),
            inbox,
            Outbox::default(),
        ))
        .id();

    // 5. Run System
    // We run it once. The worker processes ALL items in the inbox in a loop.
    schedule.run(&mut world);

    // 6. Verify Output
    let outbox = world.get::<Outbox>(entity).unwrap();
    assert_eq!(outbox.queue.len(), 4);

    let results: Vec<f64> = outbox
        .queue
        .iter()
        .map(|ticket| {
            let bytes = store.claim(ticket).unwrap();
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            json.get("rolling_mean").unwrap().as_f64().unwrap()
        })
        .collect();

    // Expected Logic:
    // Window Size = 3
    // Input: 10. State: [10]. Mean: 10
    // Input: 20. State: [10, 20]. Mean: 15
    // Input: 30. State: [10, 20, 30]. Mean: 20
    // Input: 40. State: [20, 30, 40]. Mean: 30

    assert_eq!(results, vec![10.0, 15.0, 20.0, 30.0]);
}

#[test]
fn test_window_rolling_variance() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    let (tx, _) = tokio::sync::broadcast::channel(10);
    world.insert_resource(SystemEventBus(tx));
    let store = BlobStore::new();
    world.insert_resource(store.clone());

    schedule.add_systems(window_worker);

    let node_id = uuid::Uuid::new_v4();
    let config = WindowConfig {
        target_field: "value".to_string(),
        result_key: "rolling_var".to_string(),
        operation: WindowOp::Variance,
        window_size: 3,
    };

    let node_config = NodeConfig {
        id: node_id,
        name: "Test Window Var".to_string(),
        node_type: "Window".to_string(),
        workflow_id: None,
        tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
    };

    let mut inbox = Inbox::default();

    // Inputs: 2, 4, 4, 4, 5, 5, 7, 9
    // Simplified: 2, 4, 6
    // Window 3
    // 1. [2] -> Mean 2. Var 0
    // 2. [2, 4] -> Mean 3. Var ((2-3)^2 + (4-3)^2)/2 = (1+1)/2 = 1
    // 3. [2, 4, 6] -> Mean 4. Var ((2-4)^2 + (4-4)^2 + (6-4)^2)/3 = (4+0+4)/3 = 8/3 ≈ 2.666

    let inputs = vec![2.0, 4.0, 6.0];
    for val in inputs {
        let payload = json!({ "value": val });
        let bytes = serde_json::to_vec(&payload).unwrap();
        let ticket = store.check_in(&bytes).unwrap();
        inbox.queue.push_back(ticket);
    }

    let entity = world
        .spawn((
            config,
            node_config,
            WindowState::default(),
            inbox,
            Outbox::default(),
        ))
        .id();

    schedule.run(&mut world);

    let outbox = world.get::<Outbox>(entity).unwrap();
    let results: Vec<f64> = outbox
        .queue
        .iter()
        .map(|ticket| {
            let bytes = store.claim(ticket).unwrap();
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            json.get("rolling_var").unwrap().as_f64().unwrap()
        })
        .collect();

    assert_eq!(results[0], 0.0);
    assert_eq!(results[1], 1.0);
    assert!((results[2] - 2.6666).abs() < 0.001);
}
