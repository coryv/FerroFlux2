use ferroflux_compute_wasm::components::ComputeConfig;
use ferroflux_compute_wasm::WasmComputePlugin;
use ferroflux_core::app::AppBuilder;
use ferroflux_types::{BlobStore, Inbox, NodeConfig, Outbox};
use uuid::Uuid;

#[tokio::test]
async fn test_wasm_compute_node_execution() {
    // 1. Setup App with Wasm Plugin
    let mut app_ctx = AppBuilder::new()
        .add_plugin(WasmComputePlugin)
        .build()
        .await
        .unwrap();

    // 2. Spawn a WASM Compute Node
    let node_id = Uuid::new_v4();
    let entity = app_ctx.app.world
        .spawn((
            NodeConfig {
                id: node_id,
                name: "Wasm Node".to_string(),
                node_type: "Compute".to_string(),
                workflow_id: "test_wf".to_string(),
                tenant_id: ferroflux_iam::TenantId::from("test"),
            },
            ComputeConfig {
                runtime: "js-quickjs".to_string(),
                source_code: "return { \"msg\": \"hello\" };".to_string(),
                entry_point: "main".to_string(),
            },
            Inbox {
                queue: std::collections::VecDeque::from(vec![(
                    None,
                    app_ctx
                        .app
                        .world
                        .resource::<BlobStore>()
                        .check_in(b"{\"input\": 1}")
                        .unwrap(),
                )]),
            },
            Outbox::default(),
        ))
        .id();

    // 3. Run Systems
    app_ctx.app.update();

    // 4. Verify Output
    let world = &app_ctx.app.world;
    let outbox = world.get::<Outbox>(entity).unwrap();
    assert_eq!(outbox.queue.len(), 1);

    let (_, ticket) = &outbox.queue[0];
    let store = world.resource::<BlobStore>();
    let data = store.claim(ticket).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&data).unwrap();

    assert!(json.get("processed").is_some());
    assert!(json.get("original_len").is_some());
}
