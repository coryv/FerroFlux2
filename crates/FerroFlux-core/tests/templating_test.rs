use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::components::execution_state::ActiveWorkflowState;
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
use tokio::sync::broadcast;

struct EchoTool;
impl Tool for EchoTool {
    fn id(&self) -> &'static str {
        "echo"
    }
    // We expect params to be already resolved by the engine
    fn run(&self, _ctx: &mut ToolContext, params: Value) -> anyhow::Result<Value> {
        Ok(params)
    }
}

#[test]
fn test_dataref_blob_templating() {
    let mut world = World::new();
    let store = BlobStore::default();
    world.insert_resource(store);
    world.insert_resource(SystemEventBus(broadcast::channel(10).0));

    // Add Runtime and SecretStore to satisfy pipeline requirements
    let runtime = tokio::runtime::Runtime::new().unwrap();
    world.insert_resource(ferroflux_core::resources::TokioRuntime(
        runtime.handle().clone(),
    ));

    let p_store = runtime
        .block_on(ferroflux_core::store::database::PersistentStore::new(
            "sqlite::memory:",
        ))
        .unwrap();
    let sec_store = ferroflux_core::secrets::DatabaseSecretStore::new(p_store, vec![0u8; 32]);
    world.insert_resource(sec_store);

    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(EchoTool);
    world.insert_resource(tool_registry);

    let mut def_registry = DefinitionRegistry::default();

    // Define a node that uses the "echo" tool to echo back a templated string
    let def = NodeDefinition {
        meta: NodeMeta {
            id: "templater".to_string(),
            name: "Templater".to_string(),
            node_type: "Action".to_string(),
            category: "Test".to_string(),
            version: Some("1.0".to_string()),
            description: None,
            platform: None,
            data_strategy: None,
        },
        interface: Interface {
            inputs: vec![],
            outputs: vec![],
            settings: vec![],
        },
        execution: vec![PipelineStep {
            id: "step1".to_string(),
            tool: "echo".to_string(),
            // We test two params:
            // 1. "raw": Direct access {{ large_data }} -> Should result in Blob ticket struct
            // 2. "resolved": Helper access {{ get "large_data" }} -> Should result in actual content
            params: json!({
                "raw": "{{ large_data }}",
                "resolved": "{{ get \"large_data\" }}"
            }),
            returns: HashMap::from([
                ("raw".to_string(), "res_raw".to_string()),
                ("resolved".to_string(), "res_resolved".to_string()),
            ]),
        }],
        output_transform: Some(HashMap::from([
            ("final_raw".to_string(), "steps.step1.raw".to_string()),
            (
                "final_resolved".to_string(),
                "steps.step1.resolved".to_string(),
            ),
        ])),
        context: None,
        routing: None,
    };
    def_registry
        .definitions
        .insert("templater".to_string(), def);
    world.insert_resource(def_registry);

    // Setup State with Large Blob
    let mut initial_state = ActiveWorkflowState::new();
    // Create a large object that we force into BlobStore (manually for test)
    let large_data = json!({"foo": "bar", "long_string": "a".repeat(100)});
    let store_res = world.resource::<BlobStore>();
    let ticket = store_res
        .check_in(&serde_json::to_vec(&large_data).unwrap())
        .unwrap();

    // Manually insert as Blob to bypass threshold check logic for this specific test setup
    initial_state.context.insert(
        "large_data".to_string(),
        ferroflux_core::components::execution_state::DataRef::Blob(ticket),
    );

    let state_bytes = serde_json::to_vec(&initial_state).unwrap();
    let state_ticket = store_res.check_in(&state_bytes).unwrap();

    // Spawn Node
    let mut inbox = Inbox::default();
    inbox.queue.push_back(state_ticket);

    world.spawn((
        PipelineNode {
            definition_id: "templater".to_string(),
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // Run System
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // Verify
    let mut query = world.query::<&mut Outbox>();
    let outbox = query.single(&world);
    let (_port, out_ticket) = outbox.queue.front().unwrap();

    let store_res = world.resource::<BlobStore>();
    let out_data = store_res.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    // Verification 1: "raw" ({{ large_data }}) should RESOLVE to CONTENT because of resolve_recursive optimization
    let raw_val = final_state.get("final_raw").unwrap().as_inline().unwrap();

    // It should be an OBJECT matching the original data (Auto-resolved)
    let raw_obj = raw_val
        .as_object()
        .expect("Raw (Optimized) result should be the resolved Object");
    assert_eq!(raw_obj["foo"], "bar");
    assert_eq!(raw_obj["long_string"], "a".repeat(100));

    // Verification 2: "resolved" should contain the actual content
    let resolved_val = final_state
        .get("final_resolved")
        .unwrap()
        .as_inline()
        .unwrap();
    let resolved_str = resolved_val
        .as_str()
        .expect("Resolved result should be string");

    // Note: The `get` helper outputs the JSON string representation if it's an object
    let resolved_obj: Value = serde_json::from_str(resolved_str).expect("Should be valid JSON");
    assert_eq!(resolved_obj["foo"], "bar");
    assert_eq!(resolved_obj["long_string"], "a".repeat(100));
}
