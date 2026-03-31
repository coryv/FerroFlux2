use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEventBus;
use ferroflux_core::components::execution_state::ActiveWorkflowState;
use ferroflux_core::components::pipeline::PipelineNode;
use ferroflux_core::components::{Inbox, Outbox};
use ferroflux_core::nodes::definition::{Interface, NodeDefinition, NodeMeta, PipelineStep};
use ferroflux_core::resources::DefinitionRegistry;
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::pipeline::pipeline_execution_system;
use ferroflux_core::tools::ToolRegistry;
use ferroflux_core::tools::{Tool, ToolContext};
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::sync::broadcast;

struct EchoTool;
impl Tool for EchoTool {
    fn id(&self) -> &'static str {
        "echo"
    }
    fn run(&self, _ctx: &mut ToolContext, params: Value) -> anyhow::Result<Value> {
        Ok(params)
    }
}

/// Verifies that a `DataRef::Blob` stored in the workflow context is correctly
/// resolved via a CEL expression, and that plain literals are left untouched.
///
/// This tests the lazy materialization path: the Blob is only claimed when an
/// expression that references `large_data` is evaluated.
#[test]
fn test_dataref_blob_templating() {
    let mut world = World::new();
    let store = BlobStore::default();
    world.insert_resource(store);
    world.insert_resource(SystemEventBus(broadcast::channel(10).0));

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
    world.insert_resource(ferroflux_db::oauth2::TokenRefreshLocks::default());

    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(EchoTool);
    world.insert_resource(tool_registry);

    let mut def_registry = DefinitionRegistry::default();

    // Node with three params:
    //   "data"    -- CEL identifier referencing the blob → resolves to the JSON object
    //   "as_json" -- CEL json() call → resolves to a JSON string of the object
    //   "literal" -- plain string, not valid CEL → returned as-is
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
            node_subtype: None,
            signature: None,
            permissions: vec![],
        },
        interface: Interface {
            inputs: vec![],
            outputs: vec![],
            settings: vec![],
        },
        execution: vec![PipelineStep {
            id: "step1".to_string(),
            tool: "echo".to_string(),
            params: json!({
                "data":    "large_data",
                "as_json": "json(large_data)",
                "literal": "GET",
            }),
            returns: HashMap::from([
                ("data".to_string(),    "res_data".to_string()),
                ("as_json".to_string(), "res_as_json".to_string()),
                ("literal".to_string(), "res_literal".to_string()),
            ]),
        }],
        output_transform: None,
        context: None,
        routing: None,
    };
    def_registry.definitions.insert("templater".to_string(), def);
    world.insert_resource(def_registry);

    // Store large_data as a Blob in the initial workflow state.
    let large_data = json!({"foo": "bar", "long_string": "a".repeat(100)});
    let store_res = world.resource::<BlobStore>();
    let ticket = store_res
        .check_in(&serde_json::to_vec(&large_data).unwrap())
        .unwrap();

    let mut initial_state = ActiveWorkflowState::new();
    initial_state.context.insert(
        "large_data".to_string(),
        ferroflux_core::components::execution_state::DataRef::Blob(ticket),
    );

    let state_bytes = serde_json::to_vec(&initial_state).unwrap();
    let state_ticket = store_res.check_in(&state_bytes).unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back((None, state_ticket));

    world.spawn((
        PipelineNode {
            definition_id: "templater".to_string(),
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // Read final workflow state from the outbox.
    let mut query = world.query::<&mut Outbox>();
    let outbox = query.single(&world);
    let (_port, out_ticket) = outbox.queue.front().unwrap();

    let store_res = world.resource::<BlobStore>();
    let out_data = store_res.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    // Verification 1: CEL identifier `large_data` resolves to the actual object.
    let data_val = final_state.get("res_data").unwrap().as_inline().unwrap();
    let data_obj = data_val.as_object().expect("res_data should be an object");
    assert_eq!(data_obj["foo"], "bar");
    assert_eq!(data_obj["long_string"], "a".repeat(100));

    // Verification 2: `json(large_data)` resolves to a JSON string of the object.
    let json_val = final_state.get("res_as_json").unwrap().as_inline().unwrap();
    let json_str = json_val.as_str().expect("res_as_json should be a string");
    let parsed: Value = serde_json::from_str(json_str).expect("res_as_json should be valid JSON");
    assert_eq!(parsed["foo"], "bar");
    assert_eq!(parsed["long_string"], "a".repeat(100));

    // Verification 3: Plain literal "GET" is returned unchanged.
    let literal_val = final_state.get("res_literal").unwrap().as_inline().unwrap();
    assert_eq!(literal_val.as_str().unwrap(), "GET");
}
