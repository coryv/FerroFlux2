use bevy_ecs::prelude::*;
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

struct MockTool;
impl Tool for MockTool {
    fn id(&self) -> &'static str {
        "mock_tool"
    }

    // Legacy `name` if trait still has it, but view_file showed `id` being key.
    // wait, view_file didn't show `name` in trait!
    // It showed `fn id(&self) -> &'static str;` and `fn run(...)`.
    // It did NOT show `fn name`.
    // My previous code had `fn name`. I should REMOVE `fn name`.

    fn run(&self, ctx: &mut ToolContext, _params: Value) -> anyhow::Result<Value> {
        // Mock Side Effect: Write to _outputs to simulate context enrichment
        let outputs = json!({
            "enriched_key": "enriched_value"
        });

        // We have to mutate the context map directly to simulate "emit" behavior
        // In reality, `emit` tool writes to `_outputs` key.
        ctx.local.insert(
            "_outputs".to_string(),
            ferroflux_core::components::execution_state::DataRef::Inline(outputs),
        );

        Ok(json!({"status": "ok"}))
    }
}

#[test]
fn test_unified_pipeline_execution() {
    let mut world = World::new();

    // 1. Setup Resources
    let store = BlobStore::default();
    world.insert_resource(store);

    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(MockTool);
    world.insert_resource(tool_registry);

    let mut def_registry = DefinitionRegistry::default();
    // Manually register a dummy definition so pipeline doesn't crash on lookup
    // We need to bypass `PipelineNode` looking up definition_id.
    // However, `execute_pipeline_node` checks definition_id.
    // So we must insert a valid definition.
    let def = NodeDefinition {
        meta: NodeMeta {
            id: "mock_node".to_string(),
            name: "Mock Node".to_string(),
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
            tool: "mock_tool".to_string(),
            params: json!({}),
            returns: HashMap::new(),
        }],
        output_transform: Some(HashMap::from([
            (
                "my_transformed_key".to_string(),
                "_outputs.enriched_key".to_string(),
            ), // JMESPath: select from _outputs
        ])),
        context: None,
        routing: None,
    };
    def_registry
        .definitions
        .insert("mock_node".to_string(), def);
    world.insert_resource(def_registry);

    // 2. Prepare Workflow State
    let mut initial_state = ActiveWorkflowState::new();
    initial_state.set("initial_data", json!("hello"));

    let state_bytes = serde_json::to_vec(&initial_state).unwrap();
    let store_res = world.resource::<BlobStore>();
    let ticket = store_res.check_in(&state_bytes).unwrap();

    // 3. Spawn Node Entity
    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket); // Seed with our state ticket

    world.spawn((
        PipelineNode {
            definition_id: "mock_node".to_string(),
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // 4. Run System
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);

    // 5. Verify Outbox has new state
    let mut query = world.query::<&mut Outbox>();
    let outbox = query.single(&world);
    assert_eq!(outbox.queue.len(), 1, "Outbox should have 1 ticket");

    let (_port, new_ticket) = outbox.queue.front().unwrap();

    // 6. Retrieve and Verify Content
    let store_res = world.resource::<BlobStore>();
    let new_data = store_res.claim(new_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&new_data).unwrap();

    assert_eq!(
        final_state
            .get("initial_data")
            .unwrap()
            .as_inline()
            .unwrap(),
        &json!("hello"),
        "Should preserve initial context"
    );
    assert_eq!(
        final_state
            .get("enriched_key")
            .unwrap()
            .as_inline()
            .unwrap(),
        &json!("enriched_value"),
        "Should contain enriched value from mock tool"
    );
    assert_eq!(
        final_state
            .get("my_transformed_key")
            .unwrap()
            .as_inline()
            .unwrap(),
        &json!("enriched_value"),
        "Should contain transformed key via output_transform"
    );
}

#[test]
fn test_stats_tool() {
    let mut world = World::new();

    let store = BlobStore::default();
    world.insert_resource(store);

    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(ferroflux_core::tools::primitives::StatsTool);
    world.insert_resource(tool_registry);

    let mut def_registry = DefinitionRegistry::default();
    let def = NodeDefinition {
        meta: NodeMeta {
            id: "stats_node".to_string(),
            name: "Stats Node".to_string(),
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
            tool: "ferroflux:stats".to_string(),
            // Param is "data", target_field "val".
            params: json!({
                "data": "{{ input_array }}",
                "target_field": "val",
                "enrichment_key": "statistics",
                "detect_outliers": true,
                "threshold": 1.5
            }),
            returns: HashMap::from([
                ("data".to_string(), "_state_out".to_string()), // Not using returns mapping for array output test specifically, just checking result in step? Or implicit?
            ]),
        }],
        output_transform: Some(HashMap::from([(
            "my_stats".to_string(),
            "steps.step1".to_string(),
        )])),
        context: None,
        routing: None,
    };
    def_registry
        .definitions
        .insert("stats_node".to_string(), def);
    world.insert_resource(def_registry);

    // Initial State: Array of data
    let mut initial_state = ActiveWorkflowState::new();
    let input_data = json!([
        {"val": 10.0},
        {"val": 10.0},
        {"val": 10.0},
        {"val": 100.0} // Outlier
    ]);
    initial_state.set("input_array", input_data); // Template {{ input_array }}

    let store_res = world.resource::<BlobStore>();
    let ticket = store_res
        .check_in(&serde_json::to_vec(&initial_state).unwrap())
        .unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    world.spawn((
        PipelineNode {
            definition_id: "stats_node".to_string(),
            config: HashMap::new(),
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // Run
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    // Bevy needs TokioRuntime? Tests usually mock it or don't need it if tools aren't async. StatsTool is sync.
    // However, pipeline system requests `Res<TokioRuntime>`. If it's optional in system param, fine.
    // Checked pipeline.rs: `runtime: Res<crate::resources::TokioRuntime>`. It is NOT optional in system fn signature.
    // Create Runtime First
    let runtime = tokio::runtime::Runtime::new().unwrap();
    world.insert_resource(ferroflux_core::resources::TokioRuntime(
        runtime.handle().clone(),
    ));

    // Add SystemEventBus
    world.insert_resource(ferroflux_core::api::events::SystemEventBus(
        tokio::sync::broadcast::channel(10).0,
    ));

    // Add SecretStore (Dummy) using runtime
    let p_store = runtime
        .block_on(ferroflux_core::store::database::PersistentStore::new(
            "sqlite::memory:",
        ))
        .unwrap();
    let sec_store = ferroflux_core::secrets::DatabaseSecretStore::new(p_store, vec![0u8; 32]);
    world.insert_resource(sec_store);

    // Wait, pipeline_execution_system might consume TokioRuntime? No, Res<>.
    // But duplicate add_systems call above.
    // ...
    schedule.run(&mut world);

    let mut query = world.query::<&mut Outbox>();
    let outbox = query.single(&world);
    let (_port, out_ticket) = outbox.queue.front().unwrap();

    let store_res = world.resource::<BlobStore>();
    let out_data = store_res.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    // Verify
    // output_transform mapped "steps.step1" -> "my_stats".
    // And step1 returns the enriched array "data" (because execute_pipeline_node stores tool result in steps map).

    let stats_res = final_state
        .get("my_stats")
        .unwrap()
        .as_inline()
        .unwrap()
        .as_array()
        .unwrap();

    // Check outlier (last item)
    let last_item = stats_res.last().unwrap().as_object().unwrap();
    let stats_obj = last_item
        .get("statistics")
        .expect("Stats key missing")
        .as_object()
        .expect("Stats not object");

    assert_eq!(
        stats_obj.get("is_outlier").unwrap().as_bool(),
        Some(true),
        "Should detect outlier"
    );
    // Mean of 10, 10, 10, 100 = 130 / 4 = 32.5
}
