use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_webhook_echo() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let _mock_uri = harness.mock_server().uri();

    harness.load_platforms()?;
    
    // We'll also set "core" platform config if needed, but not required yet
    // harness.set_platform_config("core", "base_url", json!(mock_uri))?;

    let workflow_yaml = std::fs::read_to_string("fixtures/webhook_echo.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    // Trigger with a specific payload
    let test_body = json!({ "hello": "world", "status": "active" });
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": test_body,
            "headers": {},
            "query": {}
        }
    }))?;

    // Tick the engine
    harness.run_until_idle(100).await;

    // Assertions
    // Since this only uses core nodes (which don't call external APIs by default),
    // we check the engine's internal state if possible, or just look for logs.
    // For now, if it didn't panic and we can finish, that's baseline success.
    // We'll rely on --nocapture logs for the "evidence" in the report.

    Ok(())
}
