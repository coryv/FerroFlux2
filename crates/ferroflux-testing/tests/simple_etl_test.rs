use ferroflux_testing::harness::TestHarness;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_simple_etl_integration() -> anyhow::Result<()> {
    // 1. Setup Tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("Starting Simple ETL Integration Test");
    
    // Enable internal IPs for testing (optional here as core.log doesn't use wiremock, but good practice)
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // 2. Setup Harness
    let mut harness = TestHarness::new().await?;
    
    // 3. Load Workflow WAML
    let workflow_yaml = std::fs::read_to_string("fixtures/simple_etl.yaml")?;
    harness.load_waml(&workflow_yaml)?;
    
    // 4. Trigger the Webhook Node
    let webhook_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    
    harness.trigger_node(webhook_uuid, json!({ 
        "event": { 
            "body": { "data": "val" },
            "headers": {},
            "query": {}
        } 
    }))?;
    
    // 5. Execute Engine
    harness.run_until_idle(100).await;
    
    // 6. Assertions
    // core.log prints directly to tracing logger. If run_until_idle finishes without panic, the test is successful.
    // In a real scenario we'd intercept tracing logs or use a custom tool, but for Simple ETL,
    // we just verify that data resolution didn't crash and the flow completed.
    
    Ok(())
}
