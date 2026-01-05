use bevy_ecs::prelude::*;
use ferroflux_core::components::execution_state::{ActiveWorkflowState, DataRef};
use ferroflux_core::components::pipeline::PipelineNode;
use ferroflux_core::components::{Inbox, Outbox};
use ferroflux_core::nodes::definition::{Interface, NodeDefinition, NodeMeta, PipelineStep};
use ferroflux_core::resources::registry::DefinitionRegistry;
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::pipeline::pipeline_execution_system;
use ferroflux_core::tools::registry::ToolRegistry;
use ferroflux_core::tools::{Tool, ToolContext};
use serde_json::{Value, json};
use std::collections::HashMap;

struct MockTool;
impl Tool for MockTool {
    fn id(&self) -> &'static str {
        "mock_tool"
    }
    // Echo functionality
    fn run(&self, _ctx: &mut ToolContext, params: Value) -> anyhow::Result<Value> {
        // Return params as result to verify resolution
        Ok(params)
    }
}

struct ErrorTool;
impl Tool for ErrorTool {
    fn id(&self) -> &'static str {
        "error_tool"
    }
    fn run(&self, _ctx: &mut ToolContext, _params: Value) -> anyhow::Result<Value> {
        Err(anyhow::anyhow!("Simulated Failure"))
    }
}

// Helper defaults locally since they aren't derived
fn default_node_meta(id: &str) -> NodeMeta {
    NodeMeta {
        id: id.to_string(),
        name: "Test Node".to_string(),
        category: "Test".to_string(),
        node_type: "action".to_string(),
        description: None,
        version: None,
        platform: None,
        data_strategy: None,
    }
}
fn default_interface() -> Interface {
    Interface {
        inputs: vec![],
        outputs: vec![],
        settings: vec![],
    }
}

async fn setup_world() -> World {
    let mut world = World::new();
    let store = BlobStore::default();
    world.insert_resource(store);

    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(MockTool);
    tool_registry.register(ErrorTool);
    world.insert_resource(tool_registry);

    let mut def_registry = DefinitionRegistry::default();

    // 1. Storage Node (Reads heavy data)
    def_registry.definitions.insert(
        "storage_node".to_string(),
        NodeDefinition {
            meta: default_node_meta("storage_node"),
            interface: default_interface(),
            execution: vec![PipelineStep {
                id: "step1".to_string(),
                tool: "mock_tool".to_string(),
                params: json!({"data": "{{ heavy_data }}"}), // Template usage
                returns: HashMap::from([("data".to_string(), "output_data".to_string())]),
            }],
            output_transform: None,
            context: None,
            routing: None,
        },
    );

    // 2. Error Node
    def_registry.definitions.insert(
        "error_node".to_string(),
        NodeDefinition {
            meta: default_node_meta("error_node"),
            interface: default_interface(),
            execution: vec![PipelineStep {
                id: "step1".to_string(),
                tool: "error_tool".to_string(),
                params: json!({}),
                returns: HashMap::new(),
            }],
            output_transform: None,
            context: None,
            routing: None,
        },
    );

    world.insert_resource(def_registry);

    // Add required resources
    world.insert_resource(ferroflux_core::api::events::SystemEventBus(
        tokio::sync::broadcast::channel(10).0,
    ));

    // Setup Secret Store
    let p_store = ferroflux_core::store::database::PersistentStore::new("sqlite::memory:")
        .await
        .unwrap();
    let sec_store = ferroflux_core::secrets::DatabaseSecretStore::new(p_store, vec![0u8; 32]);
    world.insert_resource(sec_store);

    // TokioRuntime from Handle::current()
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        world.insert_resource(ferroflux_core::resources::TokioRuntime(handle));
    }

    return world;
}

#[tokio::test]
async fn test_manifest_pattern_blob_resolution() {
    let mut world = setup_world().await;
    let store = world.resource::<BlobStore>().clone();

    // 1. Create Heavy Data and Store in BlobStore
    let heavy_payload = json!({ "large_array": [1, 2, 3, 4, 5] });
    let ticket = store
        .check_in_with_metadata(&serde_json::to_vec(&heavy_payload).unwrap(), HashMap::new())
        .unwrap();

    // 2. Create State with Reference
    let mut state = ActiveWorkflowState::new();
    state.set_ref("heavy_data", DataRef::Blob(ticket));

    let state_bytes = serde_json::to_vec(&state).unwrap();
    let state_ticket = store.check_in(&state_bytes).unwrap();

    // 3. Spawn Node
    let mut inbox = Inbox::default();
    inbox.queue.push_back(state_ticket);

    world.spawn((
        PipelineNode {
            definition_id: "storage_node".to_string(),
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));
    // Wait, query params:
    // (Entity, &mut PipelineNode, &mut Inbox, &mut Outbox, Option<&ShadowExecution>, Option<&NodeConfig>)
    // If component is missing, Option is None. I don't need to add `None` component. Bevy handles it.

    // 4. Run Tick
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // 5. Verify Output
    // The node templates `{{ heavy_data }}` into params. MockTool echoes it. Output "output_data" should be the heavy payload.
    let mut query = world.query::<&mut Outbox>();
    let outbox = query.single(&world);
    let (_port, out_ticket) = outbox.queue.front().unwrap();

    let out_data = store.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    let resolved_val = final_state.get("output_data").unwrap().as_inline().unwrap();

    // Check if it matches heavy payload
    // Note: echoing `{"data": ...}`
    assert_eq!(resolved_val.get("data").unwrap(), &heavy_payload);
}

#[tokio::test]
async fn test_quota_system() {
    let mut world = setup_world().await;
    let store = world.resource::<BlobStore>().clone();

    // Create simple state
    let state = ActiveWorkflowState::new();
    let state_bytes = serde_json::to_vec(&state).unwrap();
    let ticket = store.check_in(&state_bytes).unwrap();

    let count = 60; // > 50 (MAX_ITEMS_PER_TICK)

    let mut inbox = Inbox::default();
    for _ in 0..count {
        inbox.queue.push_back(ticket.clone());
    }

    // Spawn ONE node with 60 items in inbox
    world.spawn((
        PipelineNode {
            definition_id: "storage_node".to_string(),
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // Run Tick ONCE
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // Verify inbox still has items (10 remaining)
    let mut query = world.query::<&Inbox>();
    let inbox_after = query.single(&world);

    assert_eq!(
        inbox_after.queue.len(),
        10,
        "Quota should prevent processing all 60 items"
    );
}

#[tokio::test]
async fn test_error_handling() {
    let mut world = setup_world().await;
    let store = world.resource::<BlobStore>().clone();

    // Setup state
    let state = ActiveWorkflowState::new();
    let ticket = store
        .check_in(&serde_json::to_vec(&state).unwrap())
        .unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    world.spawn((
        PipelineNode {
            definition_id: "error_node".to_string(), // Will fail
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // Run Tick
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // Verify Output Port is `_error`
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    assert_eq!(outbox.queue.len(), 1);

    let (port, _) = outbox.queue.front().unwrap();
    assert_eq!(
        port.as_ref().unwrap(),
        "_error",
        "Should route to _error port"
    );

    // Verify state has error info
    let (_, out_ticket) = outbox.queue.front().unwrap();
    let out_data = store.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    let error_info = final_state.get("_error").unwrap().as_inline().unwrap();
    assert_eq!(error_info.get("message").unwrap(), "Simulated Failure");
}

#[tokio::test]
async fn test_memory_spill_strategy() {
    let mut world = setup_world().await;
    let store = world.resource::<BlobStore>().clone();

    // 1. Create State with a large inline value
    let mut state = ActiveWorkflowState::new();
    let large_string = "a".repeat(1500); // 1.5KB > 1KB threshold
    state.set("large_var", json!(large_string));

    // Initially it is Inline
    if let DataRef::Inline(_) = state.get("large_var").unwrap() {
        // ok
    } else {
        panic!("Should start as Inline");
    }

    // 2. Spawn a dummy node that just passes through
    // We reuse storage_node (MockTool)
    let state_bytes = serde_json::to_vec(&state).unwrap();
    let ticket = store.check_in(&state_bytes).unwrap();
    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    world.spawn((
        PipelineNode {
            definition_id: "storage_node".to_string(), // echoes inputs
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // 3. Run Pipeline System
    // This will trigger 'optimize_memory' before serialization
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // 4. Verify Output State has converted it to Blob
    let mut query = world.query::<&Outbox>();
    let outbox = query.single(&world);
    let (_, out_ticket) = outbox.queue.front().unwrap();

    let out_data = store.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    // 5. Assert it is now a Blob
    let data_ref = final_state.get("large_var").unwrap();
    if let DataRef::Blob(ticket) = data_ref {
        println!("Success: large_var was spilled to Blob: {:?}", ticket);
        // Verify content
        let blob = store.claim(&ticket).unwrap();
        let val: serde_json::Value = serde_json::from_slice(&blob).unwrap();
        assert_eq!(val, json!(large_string));
    } else {
        panic!("large_var should have been optimized to Blob!");
    }
}
