use bevy_ecs::prelude::*;
use ferroflux_core::api::events::{SystemEvent, SystemEventBus};
use ferroflux_core::components::core::{Inbox, NodeConfig};
use ferroflux_core::components::manipulation::StatsConfig;
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::manipulation::stats_worker;
use serde_json::json;
use tokio::sync::broadcast;
use uuid::Uuid;

async fn setup_world_async() -> World {
    let mut world = World::new();
    let blob_store = BlobStore::default();
    world.insert_resource(blob_store);

    let (tx, _) = broadcast::channel(100); // Increased buffer
    world.insert_resource(SystemEventBus(tx));

    world
}

#[tokio::test]
async fn test_stats_zscore_outlier() {
    let mut world = setup_world_async().await;
    let mut schedule = Schedule::default();
    schedule.add_systems(stats_worker);

    let store = world.resource::<BlobStore>().clone();
    let mut event_rx = world.resource::<SystemEventBus>().0.subscribe();

    // 1. Create Input Data - Large Dataset (20,000 items)
    // Baseline: Value = 100.0, with minor noise +/- 5.0
    // Outliers: 10000.0
    let mut data = Vec::with_capacity(20002);

    // Normal data
    for i in 0..20000 {
        // Simple deterministic pseudo-random noise
        let noise = (i % 11) - 5; // -5 to +5
        data.push(json!({ "val": 100 + noise }));
    }

    // Add Outliers
    data.push(json!({ "val": 10000 })); // Huge outlier
    data.push(json!({ "val": 5000 })); // Large outlier

    let input_json = serde_json::Value::Array(data);
    let input_bytes = serde_json::to_vec(&input_json).unwrap();
    let ticket = store.check_in(&input_bytes).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    let config = StatsConfig {
        target_field: "val".to_string(),
        enrichment_key: "stats".to_string(),
        detect_outliers: true,
        threshold: 3.0, // Standard deviation will be small (~3.0), so 3 sigma is ~109. 10000 is way out.
    };

    let node_id = Uuid::new_v4();
    world.spawn((
        NodeConfig {
            id: node_id,
            name: "Test Node".to_string(),
            node_type: "Test".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
        },
        config,
        inbox,
        ferroflux_core::components::core::Outbox::default(),
    ));

    // 2. Run System
    println!("Running Stats System on 20k items...");
    let start_time = std::time::Instant::now();
    schedule.run(&mut world);
    println!("System execution took: {:?}", start_time.elapsed());

    // 3. Verify Telemetry (wait a bit longer for large payload processing if async, though system run is sync)
    // The telemetry is emitted during execution, so it should be in channel.
    let mut processed = false;
    let timeout = tokio::time::timeout(std::time::Duration::from_millis(5000), async {
        loop {
            if let Ok(SystemEvent::NodeTelemetry {
                node_id: nid,
                node_type,
                details,
                ..
            }) = event_rx.recv().await
                && nid == node_id
                && node_type == "Stats"
            {
                println!("Telemetry: {}", details);
                let outliers = details
                    .get("outliers")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                // FIX: count is sent as f64 (20002.0)
                let count = details.get("count").and_then(|v| v.as_f64()).unwrap_or(0.0) as u64;

                assert_eq!(count, 20002, "Should process all items");
                assert!(outliers >= 2, "Should detect at least 2 outliers");
                processed = true;
                break;
            }
        }
    })
    .await;

    assert!(timeout.is_ok(), "Timed out waiting for telemetry");
    assert!(processed);

    // 4. Verify Output Data
    let mut query = world.query::<&ferroflux_core::components::core::Outbox>();
    let outbox = query.single(&world);
    assert!(!outbox.queue.is_empty());

    let (_port, out_ticket) = outbox.queue.front().unwrap();
    let out_bytes = store.claim(out_ticket).unwrap();
    let out_json: serde_json::Value = serde_json::from_slice(&out_bytes).unwrap();

    let arr = out_json.as_array().unwrap();
    assert_eq!(arr.len(), 20002);

    // Verify Outlier (Last item is 5000, Second to last is 10000)
    // Array order is preserved
    let outlier1 = &arr[20000]; // 10000
    let stats1 = outlier1.get("stats").unwrap();
    assert_eq!(
        stats1.get("is_outlier").unwrap(),
        true,
        "10000 should be outlier"
    );

    // Verify Normal Item (First item)
    let normal = &arr[0];
    let stats_norm = normal.get("stats").unwrap();
    assert_eq!(
        stats_norm.get("is_outlier").unwrap(),
        false,
        "100 should not be outlier"
    );

    println!("Verified Large Dataset Stats: 10000 -> {}", stats1);
}
