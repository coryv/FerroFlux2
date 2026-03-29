use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;
use ferroflux_integration::definition::PlatformDefinition;

#[tokio::test(flavor = "multi_thread")]
async fn test_slack_webhook_integration() -> anyhow::Result<()> {
    // 1. Setup Tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("Starting Slack Webhook Integration Test");
    
    // Enable internal IPs for testing (otherwise wiremock at 127.0.0.1 is blocked by SSRF check)
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // 2. Setup Harness
    let mut harness = TestHarness::new().await?;
    let mock_uri = harness.mock_server().uri();
    
    // 3. Mock Slack API
    // First step: conversations.open
    Mock::given(method("POST"))
        .and(path("/conversations.open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ 
            "ok": true, 
            "channel": { "id": "C12345" } 
        })))
        .mount(harness.mock_server())
        .await;

    // Second step: chat.postMessage
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(harness.mock_server())
        .await;

    println!("Slack mocks registered at {}", mock_uri);
    
    // 4. Overwrite Slack Platform base_url to mock server
    harness.set_platform_config("slack", "base_url", json!(mock_uri))?;

    // 5. Load Workflow WAML
    let workflow_yaml = std::fs::read_to_string("fixtures/slack_webhook_workflow.yaml")?;
    harness.load_waml(&workflow_yaml)?;
    
    // 6. Trigger the Webhook Node
    let webhook_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    harness.trigger_node(webhook_uuid, json!({ 
        "event": { 
            "body": { "text": "Hello from Test!" },
            "headers": {},
            "query": {}
        } 
    }))?;
    
    // 7. Execute Engine
    harness.run_until_idle(100).await;
    
    // 8. Assertions
    let received_requests = harness.mock_server().received_requests().await.expect("Mock server should have recorded requests");
    
    // We expect at least two requests: conversations.open and chat.postMessage
    assert!(received_requests.len() >= 2, "Should have received at least 2 requests to Slack API (got {})", received_requests.len());
    
    // Verify the message content
    let post_msg_req = received_requests.iter()
        .find(|r| r.url.path() == "/chat.postMessage")
        .expect("Should have called chat.postMessage");
        
    let body: serde_json::Value = serde_json::from_slice(&post_msg_req.body)?;
    assert_eq!(body["text"], "Hello from Test!");
    assert_eq!(body["channel"], "C12345");

    Ok(())
}
