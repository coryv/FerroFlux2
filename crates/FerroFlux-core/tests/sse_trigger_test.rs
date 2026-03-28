use ferroflux_connectors::components::SseTriggerConfig;
use ferroflux_connectors::ConnectorsPlugin;
use ferroflux_core::app::AppBuilder;
use ferroflux_types::resources::SseTriggerRegistry;
use ferroflux_types::{BlobStore, NodeConfig, Outbox};
use std::collections::HashMap;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_sse_trigger_lifecycle() {
    let _ = tracing_subscriber::fmt::try_init();
    
    // 1. Setup WireMock for SSE
    let mock_server = MockServer::start().await;
    
    let sse_body = "event: test_event\ndata: hello world\n\n";
    
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(sse_body)
            .append_header("Content-Type", "text/event-stream")
            .append_header("Cache-Control", "no-cache"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/events", mock_server.uri());

    // 2. Setup App with Connectors Plugin
    let mut app_ctx = AppBuilder::new()
        .add_plugin(ConnectorsPlugin)
        .build()
        .await
        .unwrap();

    // 3. Spawn SSE Trigger Node
    let node_id = Uuid::new_v4();
    let entity = app_ctx.app.world
        .spawn((
            NodeConfig {
                id: node_id,
                name: "SSE Node".to_string(),
                node_type: "SseTrigger".to_string(),
                workflow_id: "test_wf".to_string(),
                tenant_id: ferroflux_iam::TenantId::from("test"),
            },
            SseTriggerConfig {
                url,
                headers: HashMap::new(),
                reconnect_delay_ms: 100,
                max_reconnect_attempts: 1,
            },
            Outbox::default(),
        ))
        .id();
    
    {
        let mut router = app_ctx.app.world.resource_mut::<ferroflux_core::resources::NodeRouter>();
        router.0.insert(node_id, entity);
    }

    // 4. Run Systems
    let mut event_received = false;
    for i in 0..100 {
        app_ctx.app.update();
        
        // CHECK THE OUTBOX OF THE NODE INSTEAD OF GLOBAL CHANNEL
        // ingest_triggers routes from the channel TO the outbox of the entity
        if let Some(outbox) = app_ctx.app.world.get::<Outbox>(entity) {
            if let Some((_, ticket)) = outbox.queue.front() {
                println!("DEBUG: Received event in node outbox in iteration {}", i);
                let store = app_ctx.app.world.resource::<BlobStore>();
                let data = store.claim(ticket).unwrap();
                let json: serde_json::Value = serde_json::from_slice(&data).unwrap();
                
                if json["event"] == "test_event" {
                    event_received = true;
                    break;
                }
            }
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    assert!(event_received, "Failed to receive SSE event in node outbox");

    // 5. Verify Registry tracking
    {
        let registry = app_ctx.app.world.resource::<SseTriggerRegistry>();
        assert!(registry.connections.len() >= 1);
    }

    // 6. Test Shutdown
    {
        app_ctx.app.shutdown();
        let registry = app_ctx.app.world.resource::<SseTriggerRegistry>();
        assert_eq!(registry.connections.len(), 0);
    }
}
