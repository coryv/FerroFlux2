use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_webhook_http_post() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    harness.load_platforms()?;

    // Mock HTTP Sink
    Mock::given(method("POST"))
        .and(path("/post-sink"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(harness.mock_server())
        .await;

    let workflow_yaml = std::fs::read_to_string("fixtures/webhook_http_post.yaml")?;
    let workflow_yaml = workflow_yaml.replace("/post-sink", &format!("{}/post-sink", mock_uri));
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": { "data": "payload" },
            "headers": {},
            "query": {}
        }
    }))?;

    harness.run_until_idle(100).await;

    let received_requests = harness.mock_server().received_requests().await.expect("Mock server error");
    assert!(received_requests.iter().any(|r| r.url.path() == "/post-sink"), "Expected call to /post-sink");

    Ok(())
}
