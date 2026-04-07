use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_10_xml_to_json() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    
    // Load platforms
    harness.load_platforms().ok();

    // Load and trigger workflow
    let workflow_yaml = std::fs::read_to_string("fixtures/xml_to_json.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    // Test data: XML string
    let xml_input = r#"<root><item><title>FerroFlux</title></item></root>"#;
    harness.trigger_node(trigger_id, json!({ "xml": xml_input }))?;

    harness.run_until_idle(100).await;

    // Check execution
    Ok(())
}
