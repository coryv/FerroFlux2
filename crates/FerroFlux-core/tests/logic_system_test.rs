use bevy_ecs::prelude::*;
use ferroflux_core::components::{
    core::{Edge, EdgeLabel, Inbox, NodeConfig, Outbox},
    logic::{ScriptConfig, SwitchConfig},
};
use ferroflux_core::resources::WorkDone;
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::logic::{script_worker, switch_worker_safe};
use rhai::Engine;
use tokio::runtime::Runtime;

// Helper to setup world
fn setup_world() -> (World, Schedule) {
    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Resources
    world.insert_resource(BlobStore::new());
    world.insert_resource(WorkDone::default());

    // Event Bus
    let (tx, _) = tokio::sync::broadcast::channel(100);
    world.insert_resource(ferroflux_core::api::events::SystemEventBus(tx));

    // Rhai Engine
    let engine = Engine::new();
    world.insert_non_send_resource(engine);

    // Runtime
    let runtime = Runtime::new().unwrap();
    world.insert_resource(ferroflux_core::resources::TokioRuntime(
        runtime.handle().clone(),
    ));

    // Systems
    schedule.add_systems((script_worker, switch_worker_safe));

    (world, schedule)
}

#[test]
fn test_script_node_execution() {
    let (mut world, mut schedule) = setup_world();
    let store = world.resource::<BlobStore>().clone();

    // Input: Simple Number "10"
    let input = "10";
    let ticket = store.check_in(input.as_bytes()).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    // Spawn Script Node
    world.spawn((
        ScriptConfig {
            script: "let x = parse_float(input); (x * 2).to_string()".to_string(), // Manual parse if passed as string
            result_key: None,
        },
        NodeConfig {
            id: uuid::Uuid::new_v4(),
            name: "Doubler".to_string(),
            node_type: "Script".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
        },
        inbox,
        Outbox::default(),
    ));

    // Run
    schedule.run(&mut world);

    // Verify
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    assert!(!outbox.queue.is_empty());

    let (_port, result_ticket) = outbox.queue.front().unwrap();
    let data = store.claim(result_ticket).unwrap();
    let result_str = String::from_utf8(data.to_vec()).unwrap();

    // Use starts_with because there might be floating point variance "20" or "20.0"
    assert!(result_str.starts_with("20"));
}

#[test]
fn test_script_node_enrichment() {
    let (mut world, mut schedule) = setup_world();
    let store = world.resource::<BlobStore>().clone();

    // Input: JSON object context
    let input = serde_json::json!({ "original": "data" });
    let ticket = store
        .check_in(serde_json::to_vec(&input).unwrap().as_slice())
        .unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    // Spawn Script Node with result_key
    world.spawn((
        ScriptConfig {
            script: "\"enriched\"".to_string(),
            result_key: Some("new_field".to_string()),
        },
        NodeConfig {
            id: uuid::Uuid::new_v4(),
            name: "Enricher".to_string(),
            node_type: "Script".to_string(),
            workflow_id: None,
            tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
        },
        inbox,
        Outbox::default(),
    ));

    // Run
    schedule.run(&mut world);

    // Verify
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    let (_port, result_ticket) = outbox.queue.front().unwrap();
    let data = store.claim(result_ticket).unwrap();
    let result_json: serde_json::Value = serde_json::from_slice(&data).unwrap();

    assert_eq!(result_json["original"], "data");
    assert_eq!(result_json["new_field"], "enriched");
}

#[test]
fn test_switch_node_boolean_routing() {
    let (mut world, mut schedule) = setup_world();
    let store = world.resource::<BlobStore>().clone();

    // Input: "5"
    let input = "5";
    let ticket = store.check_in(input.as_bytes()).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    // Spawn Switch Node
    let switch = world
        .spawn((
            SwitchConfig {
                // switch_worker automatically parses floats
                // so input is 5.0 (float)
                script: "input > 10".to_string(), // 5 > 10 is false
            },
            NodeConfig {
                id: uuid::Uuid::new_v4(),
                name: "Switch".to_string(),
                node_type: "Switch".to_string(),
                workflow_id: None,
                tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
            },
            inbox,
        ))
        .id();

    // Spawn Targets
    let target_true = world.spawn(Inbox::default()).id();
    let target_false = world.spawn(Inbox::default()).id();

    // Spawn Edges
    world.spawn((
        Edge {
            source: switch,
            target: target_true,
            source_handle: Some("true".to_string()),
            target_handle: Some("Exec".to_string()),
        },
        EdgeLabel("true".to_string()),
    ));
    world.spawn((
        Edge {
            source: switch,
            target: target_false,
            source_handle: Some("false".to_string()),
            target_handle: Some("Exec".to_string()),
        },
        EdgeLabel("false".to_string()),
    ));

    // Run
    schedule.run(&mut world);

    // Verify
    let inbox_true = world.entity(target_true).get::<Inbox>().unwrap();
    let inbox_false = world.entity(target_false).get::<Inbox>().unwrap();

    assert!(inbox_true.queue.is_empty());
    assert_eq!(inbox_false.queue.len(), 1);
}

#[test]
fn test_switch_node_string_routing() {
    let (mut world, mut schedule) = setup_world();
    let store = world.resource::<BlobStore>().clone();

    // Input: "A"
    let input = "A";
    let ticket = store.check_in(input.as_bytes()).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    // Spawn Switch Node
    let switch = world
        .spawn((
            SwitchConfig {
                script: "input".to_string(), // Returns "A" string
            },
            NodeConfig {
                id: uuid::Uuid::new_v4(),
                name: "Switch".to_string(),
                node_type: "Switch".to_string(),
                workflow_id: None,
                tenant_id: Some(ferroflux_core::domain::TenantId::from("default_tenant")),
            },
            inbox,
        ))
        .id();

    // Spawn Targets
    let target_a = world.spawn(Inbox::default()).id();
    let target_b = world.spawn(Inbox::default()).id();

    // Spawn Edges
    world.spawn((
        Edge {
            source: switch,
            target: target_a,
            source_handle: Some("A".to_string()),
            target_handle: Some("Exec".to_string()),
        },
        EdgeLabel("A".to_string()),
    ));
    world.spawn((
        Edge {
            source: switch,
            target: target_b,
            source_handle: Some("B".to_string()),
            target_handle: Some("Exec".to_string()),
        },
        EdgeLabel("B".to_string()),
    ));

    // Run
    schedule.run(&mut world);

    // Verify
    let inbox_a = world.entity(target_a).get::<Inbox>().unwrap();
    let inbox_b = world.entity(target_b).get::<Inbox>().unwrap();

    assert_eq!(inbox_a.queue.len(), 1);
    assert!(inbox_b.queue.is_empty());
}
