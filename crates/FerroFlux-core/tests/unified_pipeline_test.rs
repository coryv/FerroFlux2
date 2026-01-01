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
        ctx.local.insert("_outputs".to_string(), outputs);

        Ok(json!({"status": "ok"}))
    }
}

#[test]
fn test_unified_pipeline_execution() {
    let mut world = World::new();

    // 1. Setup Resources
    let store = BlobStore::new();
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
    initial_state
        .context
        .insert("initial_data".to_string(), json!("hello"));

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
        final_state.context.get("initial_data").unwrap(),
        &json!("hello"),
        "Should preserve initial context"
    );
    assert_eq!(
        final_state.context.get("enriched_key").unwrap(),
        &json!("enriched_value"),
        "Should contain enriched value from mock tool"
    );
    assert_eq!(
        final_state.context.get("my_transformed_key").unwrap(),
        &json!("enriched_value"),
        "Should contain transformed key via output_transform"
    );
}
