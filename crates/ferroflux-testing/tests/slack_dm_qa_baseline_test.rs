use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;
use ferroflux_types::registry::NodeRegistry;
use ferroflux_core::resources::DefinitionRegistry;
use std::path::PathBuf;

/// Load only the slack platform directory into the harness, avoiding the broken
/// core/utils.graphql.yaml that prevents a full `load_platforms()` call.
fn load_slack_platform(harness: &mut TestHarness) -> anyhow::Result<()> {
    let mut slack_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    slack_path.pop();
    slack_path.pop();
    slack_path.push("platforms");
    slack_path.push("slack");

    let defs = {
        let mut def_registry = harness.app.world.resource_mut::<DefinitionRegistry>();
        if let Err(e) = def_registry.0.load_from_dir(&slack_path) {
            return Err(e);
        }
        def_registry.0.definitions.clone()
    };

    {
        let mut registry = harness.app.world.resource_mut::<NodeRegistry>();
        for (id, def) in &defs {
            registry.register(
                id,
                Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(
                    def.clone(),
                )),
            );
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_slack_dm_qa_baseline() -> anyhow::Result<()> {
    // 1. Setup Tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("=== QA Baseline: webhook -> slack.messages.send_dm ===");
    println!("Input payload: {{ \"user_id\": \"U99\", \"text\": \"hello world\" }}");

    // Allow wiremock's loopback address through the SSRF guard
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // 2. Setup Harness
    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    println!("Mock server listening at: {}", mock_uri);

    // 3. Register Slack API mocks
    // Step 1 of send_dm: POST /conversations.open -> returns channel id
    Mock::given(method("POST"))
        .and(path("/conversations.open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "channel": { "id": "D_U99_CHANNEL" }
        })))
        .mount(harness.mock_server())
        .await;

    // Step 2 of send_dm: POST /chat.postMessage -> returns ok
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "ts": "1234567890.123456"
        })))
        .mount(harness.mock_server())
        .await;

    println!("Mocks registered: /conversations.open, /chat.postMessage");

    // 4. Load only the Slack platform directory (avoids broken core/utils.graphql.yaml)
    load_slack_platform(&mut harness)?;
    println!("Slack platform loaded");

    // 5. Override the Slack platform base_url to point to our mock server
    harness.set_platform_config("slack", "base_url", json!(mock_uri))?;
    println!("Slack base_url overridden to: {}", mock_uri);

    // 6. Load the WAML workflow
    let workflow_yaml = std::fs::read_to_string("fixtures/slack_dm_qa_baseline.yaml")?;
    harness.load_waml(&workflow_yaml)?;
    println!("WAML workflow loaded successfully");

    // 7. Trigger the webhook node with { "user_id": "U99", "text": "hello world" }
    let webhook_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    println!("Triggering webhook node: {}", webhook_uuid);

    harness.trigger_node(webhook_uuid, json!({
        "event": {
            "body": {
                "user_id": "U99",
                "text": "hello world"
            },
            "headers": {},
            "query": {}
        }
    }))?;

    // 8. Run the engine to completion
    harness.run_until_idle(100).await;
    println!("Engine finished execution");

    // 9. Collect recorded requests
    let received_requests = harness
        .mock_server()
        .received_requests()
        .await
        .expect("Mock server should have recorded requests");

    println!("Total API calls recorded: {}", received_requests.len());

    // --- Assertion 1: both API calls were made ---
    assert!(
        received_requests.len() >= 2,
        "Expected at least 2 HTTP calls (conversations.open + chat.postMessage), got {}",
        received_requests.len()
    );
    println!("PASS: at least 2 API calls made");

    // --- Assertion 2: conversations.open was called with user_id = U99 ---
    let open_req = received_requests
        .iter()
        .find(|r| r.url.path() == "/conversations.open")
        .expect("conversations.open must have been called");

    let open_body: serde_json::Value = serde_json::from_slice(&open_req.body)?;
    println!("conversations.open request body: {}", open_body);
    assert_eq!(
        open_body["users"], "U99",
        "conversations.open should have users=U99, got: {}",
        open_body["users"]
    );
    println!("PASS: conversations.open called with users=U99");

    // --- Assertion 3: chat.postMessage was called with correct channel and text ---
    let post_req = received_requests
        .iter()
        .find(|r| r.url.path() == "/chat.postMessage")
        .expect("chat.postMessage must have been called");

    let post_body: serde_json::Value = serde_json::from_slice(&post_req.body)?;
    println!("chat.postMessage request body: {}", post_body);

    assert_eq!(
        post_body["channel"], "D_U99_CHANNEL",
        "chat.postMessage should target channel D_U99_CHANNEL (from conversations.open response), got: {}",
        post_body["channel"]
    );
    println!("PASS: chat.postMessage targeted channel D_U99_CHANNEL");

    assert_eq!(
        post_body["text"], "hello world",
        "chat.postMessage text should be 'hello world', got: {}",
        post_body["text"]
    );
    println!("PASS: chat.postMessage text = 'hello world'");

    println!("=== ALL ASSERTIONS PASSED ===");
    Ok(())
}
