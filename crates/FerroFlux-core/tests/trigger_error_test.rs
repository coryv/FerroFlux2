use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::store::BlobStore;
use ferroflux_core::traits::trigger::{TriggerEvent, TriggerProvider, TriggerSender};
use tokio::sync::broadcast;
use uuid::Uuid;

// A provider that tries to "start" (enable) but we simulate failure logic
// Since on_enable returns (), we can't test Result return.
// We'll test that it can be called.
struct MockProvider;

impl TriggerProvider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn on_enable(&self, _sender: TriggerSender) {
        // Mock startup
    }
}

#[tokio::test]
async fn test_provider_startup() {
    let (tx, _rx) = async_channel::unbounded();
    let sender = TriggerSender(tx);
    let provider = MockProvider;

    // Just verify it doesn't panic
    provider.on_enable(sender);
}

#[tokio::test]
async fn test_system_handles_unknown_trigger_type() {
    // Validates that the system ignores or logs triggers it doesn't know how to route
    let mut world = World::new();
    let (tx, rx) = async_channel::unbounded();
    world.insert_resource(ferroflux_core::traits::trigger::TriggerReceiver(rx));

    let store = BlobStore::default();
    world.insert_resource(store);

    // Dummy ticket
    let store_res = world.resource::<BlobStore>();
    let ticket = store_res.check_in(br#"{}"#).unwrap();

    // Router (Empty)
    world.insert_resource(ferroflux_core::resources::NodeRouter::default());

    // Event Bus (to capture errors/logs)
    let (bus_tx, _bus_rx) = broadcast::channel(10);
    world.insert_resource(SystemEventBus(bus_tx));

    // Add WorkDone resource (required by ingest_triggers)
    world.insert_resource(ferroflux_core::resources::WorkDone::default());

    // Send an "Unknown" trigger UUID
    tx.send(TriggerEvent {
        trigger_id: Uuid::new_v4(),
        payload: ticket,
    })
    .await
    .unwrap();

    // Run Ingestion System
    let mut schedule = Schedule::default();
    schedule.add_systems(ferroflux_core::systems::gateway::ingest_triggers);

    // Should NOT panic
    schedule.run(&mut world);

    // If it didn't panic, we assume it was dropped safely.
    assert_eq!(
        world.entities().len(),
        0,
        "No entities should be spawned for unknown trigger"
    );
}
