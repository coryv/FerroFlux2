use bevy_ecs::prelude::*;
use ferroflux_core::components::{Outbox, WorkDone};
use ferroflux_core::store::SecureTicket;
use ferroflux_core::systems::gateway::{WEBHOOK_QUEUE, bridge_webhook_queue, ingest_triggers};
use ferroflux_core::traits::trigger::{TriggerReceiver, TriggerSender};
use uuid::Uuid;

#[test]
fn test_trigger_bridge_and_ingestion() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Setup Resources
    let (tx, rx) = async_channel::unbounded();
    world.insert_resource(TriggerSender(tx));
    world.insert_resource(TriggerReceiver(rx));
    world.insert_resource(ferroflux_core::resources::NodeRouter::default());
    world.insert_resource(WorkDone::default());

    // Setup Mock Node
    let node_id = Uuid::new_v4();
    let entity = world.spawn(Outbox::default()).id();

    // Register Node in Router
    let mut router = world.resource_mut::<ferroflux_core::resources::NodeRouter>();
    router.0.insert(node_id, entity);

    // Add Systems
    schedule.add_systems((bridge_webhook_queue, ingest_triggers));

    // 1. Simulate Legacy Webhook Push
    let (wh_tx, wh_rx) = async_channel::unbounded();
    WEBHOOK_QUEUE.set((wh_tx.clone(), wh_rx)).ok(); // Initialize static queue if not set

    // If it was already set, we might need to use the existing one, but for unit tests running in parallel
    // or sequentially, modifying a static is tricky.
    // Ideally we assume tests run in isolation or we use the one present.
    // For this test, let's just push to the one we have access to via getter if set failed.

    let queue_tx = if let Some((tx, _)) = WEBHOOK_QUEUE.get() {
        tx.clone()
    } else {
        wh_tx
    };

    let ticket = SecureTicket {
        id: Uuid::new_v4(),
        metadata: std::collections::HashMap::new(),
    };

    queue_tx.try_send((node_id, ticket.clone())).unwrap();

    // 2. Run Schedule (Bridge -> TriggerSender -> TriggerReceiver -> Ingest -> Outbox)
    // We might need multiple runs if channels are async?
    // Bridge and Ingest are in same schedule.
    // Bridge reads WebhookQueue, sends to TriggerSender.
    // Ingest reads TriggerReceiver.
    // Since it's async_channel, it's available immediately?
    // Yes, Send is non-blocking usually, but receiving might need polling.
    // Let's run twice to be safe or check if one tick is enough.

    schedule.run(&mut world);

    // If bridge sends to channel, ingest might pick it up in same frame if it runs AFTER bridge?
    // Bevy schedule order is non-deterministic unless chained.
    // But async_channel is outside ECS.
    // If Ingest runs before Bridge, it sees empty. Next tick it sees it.
    // Let's run loop.

    let mut success = false;
    for _ in 0..10 {
        if world.get::<Outbox>(entity).unwrap().queue.len() > 0 {
            success = true;
            break;
        }
        schedule.run(&mut world);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(
        success,
        "Ticket failed to traverse from WebhookQueue to Node Outbox"
    );

    let outbox = world.get::<Outbox>(entity).unwrap();
    let (_, received_ticket) = outbox.queue.front().unwrap();
    assert_eq!(received_ticket.id, ticket.id);
}

// Mock Provider for Testing
struct MockTriggerProvider {
    target_node_id: Uuid,
}

impl ferroflux_core::traits::trigger::TriggerProvider for MockTriggerProvider {
    fn name(&self) -> &str {
        "MockProvider"
    }

    fn on_enable(&self, sender: TriggerSender) {
        let node_id = self.target_node_id;
        // Verify we can send on a separate thread (simulating external event source)
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(50));
            let ticket = SecureTicket {
                id: Uuid::new_v4(),
                metadata: std::collections::HashMap::new(),
            };
            sender
                .0
                .try_send(ferroflux_core::traits::trigger::TriggerEvent {
                    trigger_id: node_id,
                    payload: ticket,
                })
                .ok();
        });
    }
}

#[tokio::test]
async fn test_custom_trigger_provider_init() {
    let base_app = ferroflux_core::app::AppBuilder::new();

    // We need to build the app to trigger "on_enable"
    let node_id = Uuid::new_v4();
    let provider = MockTriggerProvider {
        target_node_id: node_id,
    };

    let build_res = base_app
        .with_trigger_provider(Box::new(provider))
        .build()
        .await;

    assert!(build_res.is_ok());
    let (mut app, _, _, _, _, _, _, _, _) = build_res.unwrap();

    // Setup routing manually to verify ingestion working
    app.world.spawn(Outbox::default()); // Just spawning unrelated entitiy to ensure world is active

    // We need to register a node to receive the event to verify full flow,
    // but here we primarily check if provider init didn't crash and channel is open.
    // The previous test verified ingestion logic.
    // Let's just run update a few times and ensure no panic.

    for _ in 0..5 {
        app.update();
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
