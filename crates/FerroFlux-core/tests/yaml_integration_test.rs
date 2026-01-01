use ferroflux_core::app::AppBuilder;
use ferroflux_core::components::Outbox;
use ferroflux_core::graph_loader::load_graph_from_str;
use ferroflux_core::store::BlobStore;
use uuid::Uuid;

#[tokio::test]
async fn test_yaml_pipeline_flow() {
    // 1. Setup App
    let (mut app, _api_tx, _event_tx, _store, _blob_store, _, _, _, _) = AppBuilder::new()
        .build()
        .await
        .expect("Failed to build app");

    app.schedule
        .add_systems(ferroflux_core::systems::pipeline::pipeline_execution_system);

    // 2. Define YAML (Manual -> Script)
    let source_id = Uuid::new_v4();
    let processor_id = Uuid::new_v4();

    // Note: We use "core.action.script" which exists in "platforms/core".
    // AppBuilder automatically loads it.
    let yaml = format!(
        r#"
nodes:
  - id: "{}"
    type: "core.trigger.manual"
    name: "Source"
    config: {{}}
  - id: "{}"
    type: "core.action.script"
    name: "Processor"
    config:
      script: |
        inputs["data"] + 10
edges:
  - source_id: "{}"
    target_id: "{}"
    source_handle: "Success"
    target_handle: "Exec"
"#,
        source_id, processor_id, source_id, processor_id
    );

    // 3. Load Graph
    load_graph_from_str(
        &mut app.world,
        ferroflux_core::domain::TenantId::from("test"),
        &yaml,
    )
    .expect("Failed to load graph");

    // 4. Trigger Source
    {
        let router = app
            .world
            .resource::<ferroflux_core::resources::NodeRouter>();
        let source_entity = router.0.get(&source_id).cloned().expect("Source not found");

        let ticket = {
            let store = app.world.resource::<BlobStore>();
            store.check_in(br#"{"inputs": {"data": 5}}"#).unwrap()
        };

        let mut outbox_q = app.world.query::<&mut Outbox>();
        let mut outbox = outbox_q
            .get_mut(&mut app.world, source_entity)
            .expect("Source Outbox missing");

        // Emitting to "Success" port to match Edge source_handle="Success"
        outbox
            .queue
            .push_back((Some("Success".to_string()), ticket));

        let processor_ent = *app
            .world
            .resource::<ferroflux_core::resources::NodeRouter>()
            .0
            .get(&processor_id)
            .unwrap();
        let _has_inbox = app
            .world
            .entity(processor_ent)
            .contains::<ferroflux_core::components::Inbox>();
    }

    // 5. Run Systems
    for _ in 0..5 {
        app.update();
    }

    // 6. Verify Processor Output
    {
        let router = app
            .world
            .resource::<ferroflux_core::resources::NodeRouter>();
        let processor_entity = router
            .0
            .get(&processor_id)
            .cloned()
            .expect("Processor not found");

        let mut outbox_q = app.world.query::<&mut Outbox>();
        let outbox = outbox_q
            .get(&app.world, processor_entity)
            .expect("Processor Outbox missing");

        assert!(
            !outbox.queue.is_empty(),
            "Processor Outbox should have tickets"
        );

        // We expect result to be present in state
        // Iterate tickets to find the one with enriched data
        let store = app.world.resource::<BlobStore>();
        let mut found_result = false;

        for (_port, ticket) in &outbox.queue {
            let data = store.claim(ticket).unwrap();
            if let Ok(state) = serde_json::from_slice::<
                ferroflux_core::components::execution_state::ActiveWorkflowState,
            >(&data)
            {
                // core.action.script defined returns: { result: script_result }
                if let Some(val) = state.context.get("result") {
                    if val.as_i64() == Some(15)
                        || val.as_u64() == Some(15)
                        || val.as_f64() == Some(15.0)
                    {
                        found_result = true;
                        break;
                    }
                }
                // Fallback: check if direct value? No, pipeline serializes ActiveWorkflowState.
            }
        }

        assert!(
            found_result,
            "Did not find expected result (15) in output state"
        );
    }
}
