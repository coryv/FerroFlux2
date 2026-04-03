use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

// Load just the Slack platform files into the harness registries,
// bypassing the broken core/utils.graphql.yaml that causes load_platforms() to bail.
fn load_slack_platform(harness: &mut TestHarness) -> anyhow::Result<()> {
    use std::path::PathBuf;
    use ferroflux_core::resources::DefinitionRegistry;
    use ferroflux_types::registry::NodeRegistry;
    use ferroflux_core::nodes::yaml_factory::YamlNodeFactory;

    let mut slack_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    slack_path.pop();
    slack_path.pop();
    slack_path.push("platforms");
    slack_path.push("slack");

    let mut def_registry = harness.app.world.resource_mut::<DefinitionRegistry>();
    if let Err(e) = def_registry.0.load_from_dir(&slack_path) {
        tracing::warn!(error = %e, "Slack platform load encountered an error");
    }
    let defs = def_registry.0.definitions.clone();
    let platforms = def_registry.0.platforms.clone();

    {
        let mut registry = harness.app.world.resource_mut::<NodeRegistry>();
        for (id, def) in &defs {
            registry.register(
                id,
                Box::new(YamlNodeFactory::new(def.clone())),
            );
        }
    }

    tracing::info!(
        "Slack platform loaded: {} nodes, {} platforms",
        defs.len(),
        platforms.len()
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_slack_dm_qa() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();

    // Load only the Slack platform directory to avoid the broken core/utils.graphql.yaml
    load_slack_platform(&mut harness)?;

    // Step 1: conversations.open — returns the DM channel ID
    Mock::given(method("POST"))
        .and(path("/conversations.open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "channel": { "id": "C99DM" }
        })))
        .mount(harness.mock_server())
        .await;

    // Step 2: chat.postMessage — 200 routes to Success
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "ts": "1234567890.123456"
        })))
        .mount(harness.mock_server())
        .await;

    harness.set_platform_config("slack", "base_url", json!(mock_uri))?;

    let workflow_yaml = std::fs::read_to_string("fixtures/slack_dm_qa.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": { "user_id": "U99", "text": "hello world" },
            "headers": {},
            "query": {}
        }
    }))?;

    harness.run_until_idle(100).await;

    let requests = harness.mock_server()
        .received_requests().await
        .expect("mock server should record requests");

    println!("Total mock HTTP requests received: {}", requests.len());
    for r in &requests {
        println!("  {} {}", r.method, r.url.path());
    }

    // Assertion 1: conversations.open was called
    assert!(
        !requests.is_empty(),
        "expected at least 1 HTTP call, got 0"
    );

    let open_req = requests.iter()
        .find(|r| r.url.path() == "/conversations.open")
        .expect("should have called POST /conversations.open");

    // Assertion 2: conversations.open body.users == "U99"
    let open_body: serde_json::Value = serde_json::from_slice(&open_req.body)?;
    println!("conversations.open body: {}", open_body);
    assert_eq!(
        open_body["users"], "U99",
        "conversations.open body.users should be U99, got: {}",
        open_body["users"]
    );

    // Assertion 3: chat.postMessage was called
    let post_req = requests.iter()
        .find(|r| r.url.path() == "/chat.postMessage")
        .expect("should have called POST /chat.postMessage");

    // Assertion 4: chat.postMessage body.text == "hello world"
    let post_body: serde_json::Value = serde_json::from_slice(&post_req.body)?;
    println!("chat.postMessage body: {}", post_body);
    assert_eq!(
        post_body["text"], "hello world",
        "chat.postMessage body.text should be 'hello world', got: {}",
        post_body["text"]
    );

    // Assertion 5: chat.postMessage body.channel == "C99DM" (from conversations.open response)
    assert_eq!(
        post_body["channel"], "C99DM",
        "chat.postMessage body.channel should be C99DM (from conversations.open), got: {}",
        post_body["channel"]
    );

    println!("All assertions passed.");
    Ok(())
}
