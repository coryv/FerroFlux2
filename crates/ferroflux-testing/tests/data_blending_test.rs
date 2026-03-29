use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_data_blending_workflow() -> anyhow::Result<()> {
    // 1. Setup Tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("Starting Data Blending Integration Test");
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // 2. Setup Harness
    let mut harness = TestHarness::new().await?;
    let mock_uri = harness.mock_server().uri();
    
    // 3. Mock Slack API - users.info
    Mock::given(method("GET"))
        .and(path("/users.info"))
        // we'll ignore query mathers for simplicity in this basic test
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ 
            "ok": true, 
            "user": { 
                "id": "U123",
                "profile": {
                    "email": "test@example.com"
                }
            } 
        })))
        .mount(harness.mock_server())
        .await;

    // 4. Mock Database API (core platform http override)
    Mock::given(method("GET"))
        .and(path("/mock_db"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ 
            "db_id": "DB999",
            "status": "active",
            "blended": true
        })))
        .mount(harness.mock_server())
        .await;

    println!("Mocks registered at {}", mock_uri);
    
    // 5. Overwrite Platform base_url to mock server
    harness.set_platform_config("slack", "base_url", json!(mock_uri.clone()))?;
    harness.set_platform_config("core", "base_url", json!(mock_uri))?; // route mock_db HTTP actions here

    // 6. Load Workflow WAML
    let workflow_yaml = std::fs::read_to_string("fixtures/data_blending.yaml")?;
    harness.load_waml(&workflow_yaml)?;
    
    // 7. Trigger the Webhook Node
    let webhook_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    harness.trigger_node(webhook_uuid, json!({ 
        "event": { 
            "body": { "user_id": "U123" },
            "headers": {},
            "query": {}
        } 
    }))?;
    
    // 8. Execute Engine
    harness.run_until_idle(100).await;
    
    // 9. Assertions
    let received_requests = harness.mock_server().received_requests().await.expect("Mock server should have recorded requests");
    
    // We expect at least two requests: users.info and mock_db
    assert!(received_requests.len() >= 2, "Should have received at least 2 requests (got {}). Engine failed to propagate data.", received_requests.len());

    Ok(())
}
