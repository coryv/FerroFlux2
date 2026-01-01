use bevy_ecs::prelude::*;
use ferroflux_core::api::events::{SystemEvent, SystemEventBus};
use ferroflux_core::components::{
    agent::{AgentConfig, OutputMode},
    core::{Inbox, NodeConfig, Outbox},
    schema::ExpectedOutput,
};
use ferroflux_core::integrations::IntegrationRegistry;
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::{agent_exec::agent_exec, agent_post::agent_post, agent_prep::agent_prep};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

fn setup_world() -> World {
    let mut world = World::new();

    // Resources
    world.insert_resource(BlobStore::new());
    world.insert_resource(ferroflux_core::resources::WorkDone::default());
    world.insert_resource(ferroflux_core::resources::AgentConcurrency(Arc::new(
        tokio::sync::Semaphore::new(10),
    )));
    world.insert_resource(ferroflux_core::resources::GlobalHttpClient::default());
    world.insert_resource(ferroflux_core::resources::templates::TemplateEngine::default());
    world.insert_resource(ferroflux_core::resources::PipelineResultChannel::default());
    let (tx, _) = tokio::sync::broadcast::channel(100);
    world.insert_resource(SystemEventBus(tx));

    // Runtime
    let runtime = Runtime::new().unwrap();
    world.insert_resource(ferroflux_core::resources::TokioRuntime(
        runtime.handle().clone(),
    ));

    // Registry (Empty is fine for "provider not found" test)
    world.insert_resource(IntegrationRegistry::default());

    // Init In-Memory DB Sync
    let store = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ferroflux_core::store::database::PersistentStore::new(
            "sqlite::memory:",
        ))
        .expect("Failed to init in-memory DB");

    let master_key = ferroflux_core::security::encryption::get_or_create_master_key().unwrap();
    world.insert_resource(store.clone()); // Insert the PersistentStore resource
    world.insert_resource(ferroflux_core::secrets::DatabaseSecretStore::new(
        store, master_key,
    ));

    world
}

#[test]
fn test_agent_missing_provider_event() {
    let mut world = setup_world();
    let mut schedule = Schedule::default();
    schedule.add_systems((agent_prep, agent_exec, agent_post));

    // Subscribe to Event Bus
    let mut rx = {
        let bus = world.resource::<SystemEventBus>();
        bus.0.subscribe()
    };

    let store = world.resource::<BlobStore>().clone();

    // 1. Create Data
    let ticket = store.check_in(b"{}").unwrap();
    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    // 2. Create Agent with INVALID provider
    let node_id = Uuid::new_v4();
    world.spawn((
        AgentConfig {
            model: "gpt-4".to_string(),
            system_instruction: "You are a test agent.".to_string(),
            provider: "invalid_provider".to_string(), // <--- ERROR SOURCE
            user_prompt_template: "{{user_prompt}}".to_string(),
            tools: vec![],
            tool_choice: ferroflux_core::components::agent::ToolChoice::Auto,
            output_mode: OutputMode::Text,
            result_key: None,
            generation_settings: ferroflux_core::components::agent::GenerationSettings::default(),
            history_config: ferroflux_core::components::agent::HistoryConfig::default(),
            connection_slug: None,
        },
        NodeConfig {
            id: node_id,
            name: "Agent".to_string(),
            node_type: "Agent".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
        },
        ExpectedOutput {
            aggregated_schema: std::collections::HashSet::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // 3. Run
    schedule.run(&mut world);

    // 4. Verify Event
    // We expect a NodeError event on the bus
    let event = rx.try_recv();
    assert!(event.is_ok(), "Should have received an event");

    if let Ok(SystemEvent::NodeError {
        node_id: nid,
        error,
        ..
    }) = event
    {
        assert_eq!(nid, node_id);
        // Note: The error message might have changed in agent logic
        // It should be "Integration 'invalid_provider' not found"
        assert!(error.contains("Integration 'invalid_provider' not found"));
    } else {
        panic!("Expected NodeError event, got {:?}", event);
    }
}
