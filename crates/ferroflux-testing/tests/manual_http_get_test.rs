use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_manual_http_get() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();

    harness.load_platforms()?;
    
    // Wire up mock server for the HTTP GET call
    Mock::given(method("GET"))
        .and(path("/get-data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ 
            "status": "success", 
            "data": "value_from_api" 
        })))
        .mount(harness.mock_server())
        .await;

    // Overwrite the core platform config so the generic HTTP node uses our mock server
    // Note: The HTTP node uses settings.url, but we can't easily prefix it in the WAML
    // unless we use a connection or the engine supports absolute URLs.
    // If the WAML says "/get-data", we should ideally be able to base it at mock_uri.
    
    // In action.http.yaml: url is settings.url.
    // Let's use an absolute URL in the WAML by injecting the mock_uri.
    
    let workflow_yaml = std::fs::read_to_string("fixtures/manual_http_get.yaml")?;
    // Replace placeholder /get-data with absolute mock URL
    let workflow_yaml = workflow_yaml.replace("/get-data", &format!("{}/get-data", mock_uri));
    
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"manual_in");
    
    // Trigger manual node
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "user_id": "test_user",
            "timestamp": "2026-04-01T21:12:00Z"
        }
    }))?;

    // Tick the engine
    harness.run_until_idle(100).await;

    // Assertions
    let received_requests = harness.mock_server().received_requests().await.expect("Mock server should have recorded requests");
    assert!(!received_requests.is_empty(), "Expected at least 1 HTTP request, got 0");
    assert_eq!(received_requests[0].url.path(), "/get-data");

    Ok(())
}
