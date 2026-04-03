use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_delay_steps() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;

    let workflow_yaml = std::fs::read_to_string("fixtures/delay_steps.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    // Trigger the workflow
    harness.trigger_node(trigger_uuid, json!({
        "event": { "body": {}, "headers": {}, "query": {} }
    }))?;

    // Wait for ticks. 5ticks might not be enough if it's sleeping.
    // Actually, harness.run_until_idle might not handle Sleep correctly (block_in_place).
    // I'll wait 500ticks or more or just one long idle.
    harness.run_until_idle(500).await;

    Ok(())
}
