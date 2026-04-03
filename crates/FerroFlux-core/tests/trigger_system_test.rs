use bevy_ecs::prelude::*;
use ferroflux_core::components::{Outbox, WorkDone};
use ferroflux_core::store::SecureTicket;
use ferroflux_core::systems::gateway::ingest_triggers;
use ferroflux_core::traits::trigger::{TriggerEvent, TriggerReceiver, TriggerSender};
use uuid::Uuid;

#[test]
fn test_trigger_ingestion() {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Setup Resources
    let (tx, rx) = async_channel::unbounded();
    world.insert_resource(TriggerSender(tx.clone()));
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
    schedule.add_systems(ingest_triggers);

    // Send trigger directly via TriggerSender
    let ticket = SecureTicket {
        id: Uuid::new_v4(),
        metadata: std::collections::HashMap::new(),
    };
    tx.try_send(TriggerEvent {
        trigger_id: node_id,
        payload: ticket.clone(),
    })
    .unwrap();

    // Run schedule until the trigger reaches the node outbox
    let mut success = false;
    for _ in 0..10 {
        schedule.run(&mut world);
        if !world.get::<Outbox>(entity).unwrap().queue.is_empty() {
            success = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(success, "Ticket failed to reach Node Outbox via TriggerSender");

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
                .try_send(TriggerEvent {
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

    let node_id = Uuid::new_v4();
    let provider = MockTriggerProvider {
        target_node_id: node_id,
    };

    let build_res = base_app
        .with_trigger_provider(Box::new(provider))
        .build()
        .await;

    assert!(build_res.is_ok());
    let mut ctx = build_res.unwrap();

    ctx.app.world.spawn(Outbox::default());

    for _ in 0..5 {
        ctx.app.update();
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
