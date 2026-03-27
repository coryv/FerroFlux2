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
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::broadcast;

#[tokio::test]
async fn test_stats_tool_large_dataset() {
    let mut world = World::new();

    // 1. Resources Setup
    // Use Handle::current() since we are in #[tokio::test]
    world.insert_resource(ferroflux_core::resources::TokioRuntime(
        tokio::runtime::Handle::current(),
    ));

    let store = BlobStore::default();
    world.insert_resource(store);

    let (tx, _) = broadcast::channel(100);
    world.insert_resource(SystemEventBus(tx));

    // Add SecretStore (Dummy) using await
    let p_store = ferroflux_core::store::database::PersistentStore::new("sqlite::memory:")
        .await
        .unwrap();
    let sec_store = ferroflux_core::secrets::DatabaseSecretStore::new(p_store, vec![0u8; 32]);
    world.insert_resource(sec_store);
    world.insert_resource(ferroflux_core::oauth2::TokenRefreshLocks::default());

    // 2. Registry Setup
    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(ferroflux_core::tools::primitives::StatsTool);
    world.insert_resource(tool_registry);

    let mut def_registry = DefinitionRegistry::default();
    // Define the Stats Node
    let def = NodeDefinition {
        meta: NodeMeta {
            id: "core.manipulation.stats".to_string(),
            name: "Statistics".to_string(),
            node_type: "Action".to_string(),
            category: "Manipulation".to_string(),
            version: Some("1.0.0".to_string()),
            description: None,
            platform: None,
            data_strategy: None,
            node_subtype: None,
        },
        interface: Interface {
            inputs: vec![],
            outputs: vec![],
            settings: vec![],
        },
        execution: vec![PipelineStep {
            id: "analyze".to_string(),
            tool: "ferroflux:stats".to_string(),
            params: json!({
                "data": "{{ inputs }}",
                "target_field": "{{ settings.target_field }}",
                "enrichment_key": "{{ settings.enrichment_key }}",
                "detect_outliers": "{{ settings.detect_outliers }}",
                "threshold": "{{ settings.threshold }}"
            }),
            returns: HashMap::from([("data".to_string(), "_state_out".to_string())]),
        }],
        output_transform: Some(HashMap::from([(
            "result".to_string(),
            "steps.analyze".to_string(),
        )])),
        context: None,
        routing: None,
    };
    def_registry
        .definitions
        .insert("core.manipulation.stats".to_string(), def);
    world.insert_resource(def_registry);

    // 3. Create Input Data - Large Dataset (20,000 items)
    let mut data = Vec::with_capacity(20002);
    for i in 0..20000 {
        let noise = (i % 11) as i64 - 5;
        data.push(json!({ "val": 100 + noise }));
    }
    // Add Outliers
    data.push(json!({ "val": 10000 }));
    data.push(json!({ "val": 5000 }));

    // Prepare ActiveWorkflowState
    let mut start_state = ActiveWorkflowState::new();
    // Inputs: direct array
    start_state.set("inputs", json!(data));

    let store_res = world.resource::<BlobStore>();
    let ticket = store_res
        .check_in(&serde_json::to_vec(&start_state).unwrap())
        .unwrap();

    let mut inbox = Inbox::default();
    inbox.queue.push_back(ticket);

    // Spawn Node Entity
    // Configuration simulates user settings
    let mut config = HashMap::new();
    config.insert("target_field".to_string(), json!("val"));
    config.insert("enrichment_key".to_string(), json!("stats"));
    config.insert("detect_outliers".to_string(), json!(true));
    config.insert("threshold".to_string(), json!(3.0));

    world.spawn((
        PipelineNode {
            definition_id: "core.manipulation.stats".to_string(),
            config,
            execution_context: HashMap::new(),
        },
        inbox,
        Outbox::default(),
    ));

    // 4. Run System
    println!("Running Pipeline with StatsTool on 20k items...");
    let start_time = std::time::Instant::now();
    let mut schedule = Schedule::default();
    schedule.add_systems(pipeline_execution_system);
    schedule.run(&mut world);
    println!("System execution took: {:?}", start_time.elapsed());

    // 5. Verify Output
    let mut query = world.query::<&mut Outbox>();
    let outbox = query.single(&world);
    assert!(!outbox.queue.is_empty(), "Outbox should have a ticket");

    let (_port, out_ticket) = outbox.queue.front().unwrap();
    let store_res = world.resource::<BlobStore>();
    let out_data = store_res.claim(out_ticket).unwrap();
    let final_state: ActiveWorkflowState = serde_json::from_slice(&out_data).unwrap();

    // Verify Result
    let result_val = final_state
        .get("result")
        .expect("Result missing from final state");

    let resolved_result = match result_val {
        ferroflux_core::components::execution_state::DataRef::Inline(v) => v.clone(),
        ferroflux_core::components::execution_state::DataRef::Blob(ticket) => world
            .resource::<BlobStore>()
            .claim(ticket)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.into()))
            .expect("Failed to claim/parse result blob"),
    };

    let result_arr = resolved_result
        .as_array()
        .expect("Result should be an array");
    assert_eq!(result_arr.len(), 20002);

    // Verify Outlier (Last item index 20001 is 5000, 20000 is 10000)
    let outlier1 = &result_arr[20000];
    let stats1 = outlier1.get("stats").unwrap();
    assert_eq!(
        stats1.get("is_outlier").unwrap(),
        true,
        "10000 should be outlier"
    );

    let normal = &result_arr[0];
    let stats_norm = normal.get("stats").unwrap();
    assert_eq!(
        stats_norm.get("is_outlier").unwrap(),
        false,
        "100 should not be outlier"
    );

    println!("Verified Large Dataset Stats: 10000 -> {}", stats1);
}
