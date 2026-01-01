use bevy_ecs::prelude::*;
use ferroflux_core::api::events::{SystemEvent, SystemEventBus};
use ferroflux_core::components::control::CheckpointConfig;
use ferroflux_core::components::core::{Inbox, NodeConfig};
use ferroflux_iam::TenantId;
use ferroflux_core::store::BlobStore;
use ferroflux_core::store::database::PersistentStore;
use ferroflux_core::systems::control::checkpoint_worker;
use tokio::sync::broadcast;
use uuid::Uuid;

// Setup helper
async fn setup_world_async() -> World {
    let mut world = World::new();
    let blob_store = BlobStore::new();
    world.insert_resource(blob_store);

    let (tx, _) = broadcast::channel(10);
    world.insert_resource(SystemEventBus(tx));

    let db_url = "sqlite::memory:";
    let store = PersistentStore::new(db_url).await.unwrap();
    // Create table if not exists (PersistentStore already does this in new())
    world.insert_resource(store);

    world
}

#[tokio::test]
async fn test_checkpoint_hibernate() {
    let mut world = setup_world_async().await;
    let mut schedule = Schedule::default();
    schedule.add_systems(checkpoint_worker);

    let store = world.resource::<BlobStore>().clone();
    // Subscribe to events
    let mut event_rx = world.resource::<SystemEventBus>().0.subscribe();

    // 1. Create Checkpoint Node
    let node_id = Uuid::new_v4();
    let config = CheckpointConfig {
        timeout_seconds: None,
    };

    let input_bytes = b"freeze_me";
    let ticket = store.check_in(input_bytes).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    world.spawn((
        NodeConfig {
            id: node_id,
            name: "Switch Node".to_string(),
            node_type: "Switch".to_string(),
            workflow_id: None,
            tenant_id: Some(TenantId::from("default_tenant")),
        },
        config,
        inbox,
    ));

    // 2. Run System
    schedule.run(&mut world);

    // 3. Verify Event (Wait for async spawn)
    // We expect CheckpointCreated
    let mut received_token = None;

    // Allow some time for the spawned task to complete
    let timeout = tokio::time::timeout(std::time::Duration::from_millis(500), async {
        loop {
            if let Ok(SystemEvent::CheckpointCreated {
                token,
                node_id: _,
                trace_id: _,
            }) = event_rx.recv().await
            {
                received_token = Some(token);
                break;
            }
        }
    })
    .await;

    assert!(
        timeout.is_ok(),
        "Timed out waiting for CheckpointCreated event"
    );
    assert!(received_token.is_some(), "Did not receive checkpoint token");

    let token = received_token.unwrap();
    println!("Checkpoint Token: {}", token);

    // 4. Verify DB has checkpoint
    let db = world.resource::<PersistentStore>();
    // Use claim to verify it exists (consume-on-read)
    let tenant_id = TenantId::from("default_tenant");
    let result = db
        .claim_checkpoint(&tenant_id, &token)
        .await
        .expect("DB Claim failed");
    assert!(result.is_some(), "Checkpoint should be claimable from DB");
    let (claimed_node_id, claimed_data, _) = result.unwrap();

    assert_eq!(claimed_node_id, node_id);
    assert_eq!(claimed_data, input_bytes);

    // 5. Verify it is gone after claim
    let result_again = db
        .claim_checkpoint(&tenant_id, &token)
        .await
        .expect("DB Claim 2 failed");
    assert!(
        result_again.is_none(),
        "Checkpoint should be gone after claim"
    );
}
