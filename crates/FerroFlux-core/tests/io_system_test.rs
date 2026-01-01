use bevy_ecs::prelude::*;
use ferroflux_core::components::{
    core::{Inbox, NodeConfig, Outbox},
    io::HttpConfig,
};
use ferroflux_core::resources::WorkDone;
use ferroflux_core::store::BlobStore;
use ferroflux_core::systems::io::http_worker;
use std::env;
use std::time::Duration;
use tokio::runtime::Runtime;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper to setup world
async fn setup_world() -> (World, Schedule) {
    // Enable internal IP access for tests
    unsafe {
        env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true");
    }

    let mut world = World::new();
    let mut schedule = Schedule::default();

    // Resources
    world.insert_resource(BlobStore::new());
    world.insert_resource(WorkDone::default());

    // Event Bus
    let (tx, _) = tokio::sync::broadcast::channel(100);
    world.insert_resource(ferroflux_core::api::events::SystemEventBus(tx));
    // Insert the new HttpResultChannel resource
    world.insert_resource(ferroflux_core::resources::HttpResultChannel::default());

    // Insert DatabaseSecretStore (In-Memory)
    let store = ferroflux_core::store::database::PersistentStore::new("sqlite::memory:")
        .await
        .expect("Failed to init in-memory DB");
    let master_key = ferroflux_core::security::encryption::get_or_create_master_key()
        .expect("Failed to get master key");
    world.insert_resource(ferroflux_core::secrets::DatabaseSecretStore::new(
        store, master_key,
    ));

    // Insert TokioRuntime
    let handle = tokio::runtime::Handle::current();
    world.insert_resource(ferroflux_core::resources::TokioRuntime(handle));

    // NOTE: Runtime is NOT inserted here to avoid "Cannot drop a runtime..." panic.
    // The test wrapper `rt.block_on` provides the async context.

    // Systems
    schedule.add_systems(http_worker);

    (world, schedule)
}

#[test]
fn test_http_worker_get_success() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_server = MockServer::start().await;
        // setup_world is async here
        let (mut world, mut schedule) = setup_world().await;

        Mock::given(method("GET"))
            .and(path("/data"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let store = world.resource::<BlobStore>().clone();

        // Input Ticket (Empty)
        let ticket = store.check_in(b"{}").unwrap();
        let mut inbox = Inbox::default();
        inbox.queue.push_back(ticket);

        // Spawn Http Node
        let node_id = uuid::Uuid::new_v4();
        world.spawn((
            HttpConfig {
                url: format!("{}/data", mock_server.uri()),
                method: "GET".to_string(),
                result_key: None,
                connection_slug: None,
            },
            NodeConfig {
                id: node_id,
                name: "Fetcher".to_string(),
                node_type: "Http".to_string(),
                workflow_id: None,
                tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
            },
            inbox,
            Outbox::default(),
        ));

        // Wait for result
        let mut success = false;
        for _ in 0..50 {
            schedule.run(&mut world);

            let ticket = {
                let mut query = world.query::<&Outbox>();
                let outbox = query.get_single(&world).ok();
                outbox.and_then(|o| o.queue.front().map(|(_port, t)| t.clone()))
            };

            if let Some(t) = ticket {
                let data = store.claim(&t).unwrap();
                let output = String::from_utf8(data.to_vec()).unwrap();
                assert_eq!(output, "OK");
                success = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        assert!(success, "Http worker timed out");
    });
}

#[test]
fn test_http_worker_post_enrichment() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mock_server = MockServer::start().await;
        let (mut world, mut schedule) = setup_world().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 123})))
            .mount(&mock_server)
            .await;

        let store = world.resource::<BlobStore>().clone();

        // Input Payload
        let input = serde_json::json!({"context": "test"});
        let ticket = store
            .check_in(serde_json::to_vec(&input).unwrap().as_slice())
            .unwrap();
        let mut inbox = Inbox::default();
        inbox.queue.push_back(ticket);

        // Spawn Http Node
        world.spawn((
            HttpConfig {
                url: mock_server.uri(),
                method: "POST".to_string(),
                result_key: Some("api_response".to_string()),
                connection_slug: None,
            },
            NodeConfig {
                id: uuid::Uuid::new_v4(),
                name: "Poster".to_string(),
                node_type: "Http".to_string(),
                workflow_id: None,
                tenant_id: Some(ferroflux_iam::TenantId::from("default_tenant")),
            },
            inbox,
            Outbox::default(),
        ));

        // Wait for result
        let mut success = false;
        for _ in 0..50 {
            schedule.run(&mut world);

            let ticket = {
                let mut query = world.query::<&Outbox>();
                let outbox = query.get_single(&world).ok();
                outbox.and_then(|o| o.queue.front().map(|(_port, t)| t.clone()))
            };

            if let Some(t) = ticket {
                let data = store.claim(&t).unwrap();
                let output: serde_json::Value = serde_json::from_slice(&data).unwrap();

                assert_eq!(output["context"], "test");

                // Correct assertion: Check the JSON property, not string containment
                assert_eq!(output["api_response"]["id"], 123);

                success = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        assert!(success, "Http worker timed out");
    });
}
