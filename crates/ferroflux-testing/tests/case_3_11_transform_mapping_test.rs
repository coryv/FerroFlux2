use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_11_transform_mapping() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms().ok();

    // Load and trigger workflow
    let workflow_yaml = std::fs::read_to_string("fixtures/transform_mapping.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    // Test data: User schema
    harness.trigger_node(trigger_id, json!({
        "first_name": "Jane",
        "last_name": "Doe",
        "email": "jane@example.com"
    }))?;

    harness.run_until_idle(100).await;

    Ok(())
}
