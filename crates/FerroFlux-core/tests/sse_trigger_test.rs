use bevy_ecs::prelude::*;
use ferroflux_core::components::connectors::SseTriggerConfig;
use ferroflux_core::components::{NodeConfig, Outbox, WorkDone};
use ferroflux_core::resources::sse_registry::SseTriggerRegistry;
use ferroflux_core::resources::{NodeRouter, TokioRuntime};
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::connectors::sse::sse_trigger_system;
use ferroflux_core::traits::trigger::{TriggerReceiver, TriggerSender};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::test]
async fn test_sse_trigger_lifecycle() {
    // 1. Setup a mock SSE server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let response = "HTTP/1.1 200 OK\r\n\
                            Content-Type: text/event-stream\r\n\
                            Cache-Control: no-cache\r\n\
                            Connection: keep-alive\r\n\r\n\
                            event: test_event\n\
                            data: hello world\n\n";
            let _ = socket.write_all(response.as_bytes()).await;
            // Keep connection open for a bit
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    });

    // 2. Setup Bevy World
    let mut world = World::new();
    let mut schedule = Schedule::default();

    let (tx, rx) = async_channel::unbounded();
    world.insert_resource(TriggerSender(tx));
    world.insert_resource(TriggerReceiver(rx));
    world.insert_resource(BlobStore::default());
    world.insert_resource(NodeRouter::default());
    world.insert_resource(WorkDone::default());
    world.insert_resource(TokioRuntime(tokio::runtime::Handle::current()));
    world.insert_resource(SseTriggerRegistry::default());

    // 3. Spawn SSE Trigger Node
    let node_id = Uuid::new_v4();
    let entity = world
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

    // Register in router (so ingest_triggers can route it, though we'll check TriggerReceiver directly)
    world.resource_mut::<NodeRouter>().0.insert(node_id, entity);

    // 4. Run System
    schedule.add_systems(sse_trigger_system);
    
    // Run multiple times to ensure connection is spawned and event processed
    let mut event_received = false;
    for _ in 0..10 {
        schedule.run(&mut world);
        
        let rx = world.resource::<TriggerReceiver>();
        if let Ok(event) = rx.0.try_recv() {
            assert_eq!(event.trigger_id, node_id);
            
            // Validate payload content
            let store = world.resource::<BlobStore>();
            let data = store.claim(&event.payload).unwrap();
            let json: serde_json::Value = serde_json::from_slice(&data).unwrap();
            
            assert_eq!(json["event"], "test_event");
            assert_eq!(json["data"], "hello world");
            
            event_received = true;
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert!(event_received, "Failed to receive SSE event");

    // 5. Verify Registry tracking
    {
        let registry = world.resource::<SseTriggerRegistry>();
        assert_eq!(registry.connections.len(), 1);
        let key = ("test_wf".to_string(), node_id);
        assert!(registry.connections.contains_key(&key));
    }

    // 6. Test Shutdown
    {
        let mut registry = world.resource_mut::<SseTriggerRegistry>();
        registry.abort_all();
        assert_eq!(registry.connections.len(), 0);
    }
}
