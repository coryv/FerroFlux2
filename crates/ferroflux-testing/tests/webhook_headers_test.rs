use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path, header};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_webhook_headers() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    harness.load_platforms()?;

    // Mock HTTP Sink with header matchers
    Mock::given(method("GET"))
        .and(path("/headers-sink"))
        .and(header("Authorization", "Bearer test-token"))
        .and(header("X-Test", "Value"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .mount(harness.mock_server())
        .await;

    let workflow_yaml = std::fs::read_to_string("fixtures/webhook_headers.yaml")?;
    let workflow_yaml = workflow_yaml.replace("/headers-sink", &format!("{}/headers-sink", mock_uri));
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": {},
            "headers": {},
            "query": {}
        }
    }))?;

    harness.run_until_idle(100).await;

    let received_requests = harness.mock_server().received_requests().await.expect("Mock server error");
    let req = received_requests.iter().find(|r| r.url.path() == "/headers-sink").expect("No request to /headers-sink");
    
    let auth_header = req.headers.get("Authorization").and_then(|v| v.to_str().ok());
    let test_header = req.headers.get("X-Test").and_then(|v| v.to_str().ok());

    assert_eq!(auth_header, Some("Bearer test-token"), "Missing or incorrect Authorization header");
    assert_eq!(test_header, Some("Value"), "Missing or incorrect X-Test header");

    Ok(())
}
