use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_sse_trigger() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;

    let workflow_yaml = std::fs::read_to_string("fixtures/sse_to_log.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"sse_in");
    
    // Simulate SSE event
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "data": { "msg": "hello sse" },
            "raw": "data: {\"msg\": \"hello sse\"}",
            "event_type": "message",
            "id": "sse-1"
        }
    }))?;

    harness.run_until_idle(100).await;

    Ok(())
}
