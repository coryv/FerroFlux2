use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

/// QA Baseline: webhook -> slack.action.users.get -> slack.messages.send_dm
///
/// Verifies that the user object returned by users.get (step 1) propagates
/// correctly into the send_dm call (step 2): the DM is opened for the user id
/// returned by the API, and the message text contains that user's real_name.
#[tokio::test(flavor = "multi_thread")]
async fn test_two_step_propagation_qa_baseline() -> anyhow::Result<()> {
    // ── 1. Tracing ───────────────────────────────────────────────────────────
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("=== QA Baseline: Two-Step Propagation Test ===");

    // Allow wiremock's 127.0.0.1 address through the SSRF guard.
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // ── 2. Harness ───────────────────────────────────────────────────────────
    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();

    // ── 3. HTTP Mocks ────────────────────────────────────────────────────────
    // HTTP Call 1 (from slack.action.users.get): GET /users.info?user=U42
    // Returns a user object with id and real_name that step 2 will consume.
    Mock::given(method("GET"))
        .and(path("/users.info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "user": {
                "id": "U42",
                "real_name": "Alice Example",
                "profile": {
                    "email": "alice@example.com"
                }
            }
        })))
        .mount(harness.mock_server())
        .await;

    // HTTP Call 2 (from slack.messages.send_dm, step open_dm): POST /conversations.open
    Mock::given(method("POST"))
        .and(path("/conversations.open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true,
            "channel": { "id": "DM_CHANNEL_42" }
        })))
        .mount(harness.mock_server())
        .await;

    // HTTP Call 3 (from slack.messages.send_dm, step send_message): POST /chat.postMessage
    Mock::given(method("POST"))
        .and(path("/chat.postMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(harness.mock_server())
        .await;

    println!("Mocks registered at {}", mock_uri);

    // ── 4. Platform Config ───────────────────────────────────────────────────
    // Load all platforms from the platforms/ directory so "slack" is registered.
    harness.load_platforms()?;
    harness.set_platform_config("slack", "base_url", json!(mock_uri))?;

    // ── 5. Load WAML ─────────────────────────────────────────────────────────
    let yaml_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/two_step_propagation_qa_baseline.yaml");
    let workflow_yaml = std::fs::read_to_string(&yaml_path)?;
    println!("Loaded WAML:\n{}", workflow_yaml);
    harness.load_waml(&workflow_yaml)?;

    // ── 6. Trigger ───────────────────────────────────────────────────────────
    // The webhook fires with { "user_id": "U42" } as described in the QA spec.
    let webhook_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(webhook_uuid, json!({ "user_id": "U42" }))?;
    println!("Webhook triggered with payload: {{ \"user_id\": \"U42\" }}");

    // ── 7. Run ───────────────────────────────────────────────────────────────
    harness.run_until_idle(200).await;

    // ── 8. Assertions ────────────────────────────────────────────────────────
    let requests = harness
        .mock_server()
        .received_requests()
        .await
        .expect("Mock server should have recorded requests");

    println!("\n--- Received {} HTTP request(s) ---", requests.len());
    for (i, req) in requests.iter().enumerate() {
        println!(
            "  [{}] {} {}",
            i + 1,
            req.method,
            req.url.path()
        );
        if let Ok(body_str) = std::str::from_utf8(&req.body)
            && !body_str.is_empty() {
                println!("      body: {}", body_str);
        }
        if let Some(query) = req.url.query() {
            println!("      query: {}", query);
        }
    }

    // ── Assertion A: All three HTTP calls were made ───────────────────────────
    assert!(
        requests.len() >= 3,
        "Expected at least 3 HTTP calls (users.info, conversations.open, chat.postMessage) but got {}",
        requests.len()
    );

    // ── Assertion B: users.info was called with user_id from webhook ──────────
    let users_info_req = requests
        .iter()
        .find(|r| r.url.path() == "/users.info")
        .expect("Should have called GET /users.info");

    let query = users_info_req.url.query().unwrap_or("");
    assert!(
        query.contains("user=U42"),
        "users.info query should contain user=U42 (got: {:?})",
        query
    );
    println!("\nAssertion A PASS: GET /users.info called with user=U42");

    // ── Assertion C: conversations.open was called with the user id from the ──
    //                user object returned by users.get (data propagation check)
    let conv_open_req = requests
        .iter()
        .find(|r| r.url.path() == "/conversations.open")
        .expect("Should have called POST /conversations.open");

    let conv_body: serde_json::Value = serde_json::from_slice(&conv_open_req.body)
        .expect("conversations.open body should be valid JSON");
    assert_eq!(
        conv_body["users"], "U42",
        "conversations.open should use the user.id ('U42') from the users.get response. Got: {:?}",
        conv_body["users"]
    );
    println!("Assertion B PASS: POST /conversations.open called with users=U42 (propagated from step 1)");

    // ── Assertion D: chat.postMessage used DM channel from conversations.open ─
    let post_msg_req = requests
        .iter()
        .find(|r| r.url.path() == "/chat.postMessage")
        .expect("Should have called POST /chat.postMessage");

    let post_body: serde_json::Value = serde_json::from_slice(&post_msg_req.body)
        .expect("chat.postMessage body should be valid JSON");
    assert_eq!(
        post_body["channel"], "DM_CHANNEL_42",
        "chat.postMessage should use channel id from conversations.open response. Got: {:?}",
        post_body["channel"]
    );
    println!("Assertion C PASS: POST /chat.postMessage used channel=DM_CHANNEL_42 (from open_dm step)");

    // ── Assertion E: message text contains real_name from user object ─────────
    // This is the key cross-node propagation check.
    let text = post_body["text"].as_str().unwrap_or("");
    assert!(
        text.contains("Alice Example"),
        "chat.postMessage text should contain real_name 'Alice Example' propagated from users.get. Got: {:?}",
        text
    );
    println!("Assertion D PASS: message text contains 'Alice Example' (real_name propagated from users.get user object)");

    println!("\n=== QA Baseline PASSED ===");
    Ok(())
}
