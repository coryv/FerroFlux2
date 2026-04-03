use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_split_aggregate() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;

    let workflow_yaml = std::fs::read_to_string("fixtures/split_aggregate.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    // Trigger fan-out fan-in
    harness.trigger_node(trigger_uuid, json!({
        "event": { "body": {}, "headers": {}, "query": {} }
    }))?;

    // We process 5 items, each needing a separate node run for 'agg' and 'log'.
    // Synchronization Gate + Persistent Memory should handle it.
    harness.run_until_idle(1000).await;

    Ok(())
}
