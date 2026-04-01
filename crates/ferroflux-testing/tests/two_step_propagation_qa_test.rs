use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;
use ferroflux_core::resources::DefinitionRegistry;
use ferroflux_integration::definition::{NodeDefinition, PlatformDefinition};
use ferroflux_types::registry::NodeRegistry;

/// QA Test: webhook → slack.action.users.get → slack.messages.send_dm
///
/// Verifies that:
///  1. users.get calls GET /users.info with query ?user=U42
///  2. The user object emitted on the "user" output port propagates into send_dm
///  3. send_dm calls POST /conversations.open (indicating it received the user_id input)
///  4. send_dm calls POST /chat.postMessage with correct channel and text
#[tokio::test(flavor = "multi_thread")]
async fn test_two_step_propagation() -> anyhow::Result<()> {
    // 1. Setup tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("=== Two-Step Propagation QA Test ===");
    println!("Workflow: webhook -> slack.action.users.get -> slack.messages.send_dm");

    // Allow wiremock (internal IP) to pass SSRF check
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // 2. Setup harness
    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();

    // 3. Register mock HTTP stubs

    // Step 1: slack.action.users.get -> GET /users.info?user=U42
    Mock::given(method("GET"))
        .and(path("/users.info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "user": {
                "id": "U42",
                "name": "testuser",
                "profile": {
                    "email": "testuser@example.com",
                    "real_name": "Test User"
                }
            }
        })))
        .mount(harness.mock_server())
        .await;

    // Step 2a: slack.messages.send_dm -> POST /conversations.open
    Mock::given(method("POST"))
        .and(path("/conversations.open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "channel": { "id": "C_PROPAGATED" }
        })))
        .mount(harness.mock_server())
        .await;

    // Step 2b: slack.messages.send_dm -> POST /chat.postMessage
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "ts": "1234567890.000001"
        })))
        .mount(harness.mock_server())
        .await;

    println!("Mock stubs registered at {}", mock_uri);

    // 4. Inject the Slack platform definition manually
    //    We use add_mocked_integration to bypass the broken core/utils.graphql.yaml
    //    that prevents load_platforms() from completing.
    let slack_platform_yaml = format!(r#"
meta:
  permissions: ["network:api.slack.com"]
  id: slack
  name: Slack
  type: Platform
  category: Communication
  description: "Slack platform for testing"
  version: "1.0.0"
config:
  base_url: "{mock_uri}"
  headers:
    Content-Type: "application/json"
    Authorization: "Bearer TEST_TOKEN"
settings:
  - name: api_key
    label: "Bot Token"
    type: string
    required: true
"#, mock_uri = mock_uri);

    let platform_def: PlatformDefinition = serde_yaml::from_str(&slack_platform_yaml)?;
    {
        let mut def_registry = harness.app.world.resource_mut::<DefinitionRegistry>();
        def_registry.0.platforms.insert("slack".to_string(), platform_def);
    }
    println!("Slack platform registered pointing at {}", mock_uri);

    // 5. Inject the node definitions for users.get and send_dm
    //    Read the real platform YAML files and parse them as NodeDefinitions.
    let users_get_yaml = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../platforms/slack/action.users.get.yaml")
    )?;
    let send_dm_yaml = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../platforms/slack/action.messages.send_dm.yaml")
    )?;

    let users_get_def: NodeDefinition = serde_yaml::from_str(&users_get_yaml)?;
    let send_dm_def: NodeDefinition = serde_yaml::from_str(&send_dm_yaml)?;

    println!("Loaded node definitions: {} and {}", users_get_def.meta.id, send_dm_def.meta.id);

    // Register factories in NodeRegistry and definitions in DefinitionRegistry
    {
        let mut def_registry = harness.app.world.resource_mut::<DefinitionRegistry>();
        def_registry.0.definitions.insert(users_get_def.meta.id.clone(), users_get_def.clone());
        def_registry.0.definitions.insert(send_dm_def.meta.id.clone(), send_dm_def.clone());
    }
    {
        let mut node_registry = harness.app.world.resource_mut::<NodeRegistry>();
        node_registry.register(
            &users_get_def.meta.id.clone(),
            Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(users_get_def)),
        );
        node_registry.register(
            &send_dm_def.meta.id.clone(),
            Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(send_dm_def)),
        );
    }
    println!("Node definitions and factories registered");

    // 6. Load the WAML workflow fixture
    let workflow_yaml = std::fs::read_to_string("fixtures/two_step_propagation_qa.yaml")?;
    println!("Loaded WAML fixture:\n{}", workflow_yaml);
    harness.load_waml(&workflow_yaml)?;

    // 7. Trigger the webhook node
    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    println!("Triggering webhook node UUID: {}", trigger_uuid);

    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": { "user_id": "U42" },
            "headers": {},
            "query": {}
        }
    }))?;

    // 8. Run engine until idle
    harness.run_until_idle(100).await;

    // 9. Collect all HTTP requests received by mock server
    let received_requests = harness
        .mock_server()
        .received_requests()
        .await
        .expect("mock server should record requests");

    println!("=== Received {} HTTP request(s) ===", received_requests.len());
    for (i, req) in received_requests.iter().enumerate() {
        let body_str = std::str::from_utf8(&req.body).unwrap_or("<non-utf8>");
        println!("  Request {}: {} {} | body: {}", i + 1, req.method, req.url, body_str);
    }

    // --- ASSERTION 1: users.info was called ---
    let users_info_req = received_requests
        .iter()
        .find(|r| r.url.path() == "/users.info");

    assert!(
        users_info_req.is_some(),
        "FAIL: Expected GET /users.info to be called — slack.action.users.get did not execute. \
        The engine may not have propagated the flow signal from webhook to get_user node."
    );
    println!("PASS: GET /users.info called");

    // --- ASSERTION 2: users.info query param user=U42 ---
    let users_info_req = users_info_req.unwrap();
    let query = users_info_req.url.query().unwrap_or("");
    assert!(
        query.contains("user=U42"),
        "FAIL: /users.info query should contain 'user=U42' (from config.user_id), got: '{}'",
        query
    );
    println!("PASS: /users.info called with correct query param user=U42");

    // --- ASSERTION 3: conversations.open was called (data propagation gate) ---
    // This is the critical propagation check. send_dm only executes if it received
    // a flow signal from users.get's "Success" port AND its execution ran.
    let conv_open_req = received_requests
        .iter()
        .find(|r| r.url.path() == "/conversations.open");

    assert!(
        conv_open_req.is_some(),
        "FAIL: Expected POST /conversations.open to be called — slack.messages.send_dm did not execute. \
        This indicates the engine did NOT propagate the flow or data from users.get to send_dm. \
        The 'user' object from users.get's output port failed to reach send_dm's user_id input."
    );
    println!("PASS: POST /conversations.open called — data propagated from users.get to send_dm");

    // --- ASSERTION 4: conversations.open body — user_id propagation check ---
    let conv_open_req = conv_open_req.unwrap();
    let conv_body: serde_json::Value = serde_json::from_slice(&conv_open_req.body)
        .unwrap_or(json!({}));
    println!("conversations.open request body: {}", conv_body);

    // body.users should be non-null: it is inputs.user_id in the platform YAML.
    // If the "user" port from users.get was correctly wired to user_id of send_dm,
    // this field will contain either the user ID string or the propagated user object.
    let users_field = &conv_body["users"];
    assert!(
        !users_field.is_null(),
        "FAIL: POST /conversations.open body.users is null. \
        The user_id input of send_dm was not populated from the 'user' output of users.get."
    );
    println!("PASS: /conversations.open body.users is present: {}", users_field);

    // --- ASSERTION 5: chat.postMessage was called ---
    let post_msg_req = received_requests
        .iter()
        .find(|r| r.url.path() == "/chat.postMessage");

    assert!(
        post_msg_req.is_some(),
        "FAIL: Expected POST /chat.postMessage to be called — send_dm did not complete its second HTTP step. \
        conversations.open ran but the channel ID was not passed to postMessage."
    );
    println!("PASS: POST /chat.postMessage called");

    // --- ASSERTION 6: chat.postMessage body.channel comes from conversations.open response ---
    let post_msg_req = post_msg_req.unwrap();
    let msg_body: serde_json::Value = serde_json::from_slice(&post_msg_req.body)
        .unwrap_or(json!({}));
    println!("chat.postMessage request body: {}", msg_body);

    assert_eq!(
        msg_body["channel"], "C_PROPAGATED",
        "FAIL: chat.postMessage body.channel should be 'C_PROPAGATED' (the channel.id returned \
        by the mocked conversations.open response), got: {}",
        msg_body["channel"]
    );
    println!("PASS: chat.postMessage body.channel == 'C_PROPAGATED' (resolved from conversations.open response)");

    // --- ASSERTION 7: chat.postMessage body.text matches workflow config ---
    assert_eq!(
        msg_body["text"], "Hello from propagation test",
        "FAIL: chat.postMessage body.text should be 'Hello from propagation test' (from WAML config), got: {}",
        msg_body["text"]
    );
    println!("PASS: chat.postMessage body.text == 'Hello from propagation test'");

    println!("=== All 7 assertions passed ===");
    Ok(())
}
