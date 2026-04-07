use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_14_pdf_email() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms().ok();

    // Mock Resend API
    let resend_yaml = std::fs::read_to_string("../../platforms/resend/resend.yaml")?;
    harness.add_mocked_integration(&resend_yaml)?;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "e_1234"
        })))
        .mount(harness.mock_server())
        .await;

    // Load and trigger workflow
    let workflow_yaml = std::fs::read_to_string("fixtures/pdf_email.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    // Trigger with report data
    harness.trigger_node(trigger_id, json!({
        "report_data": "FerroFlux Performance Report Q1"
    }))?;

    harness.run_until_idle(100).await;

    // Verify mock request
    let requests = harness.mock_server().received_requests().await.unwrap();
    assert!(!requests.is_empty(), "expected at least 1 Resend request");

    Ok(())
}
