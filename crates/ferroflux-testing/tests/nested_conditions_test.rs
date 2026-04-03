use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_nested_conditions() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;

    let workflow_yaml = std::fs::read_to_string("fixtures/nested_conditions.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");

    println!(">>> CASE 1: High value order");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": { "type": "order", "value": 1000 },
            "headers": {},
            "query": {}
        }
    }))?;
    harness.run_until_idle(100).await;

    println!(">>> CASE 2: Low value order");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": { "type": "order", "value": 200 },
            "headers": {},
            "query": {}
        }
    }))?;
    harness.run_until_idle(100).await;

    println!(">>> CASE 3: Not an order");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": { "type": "subscription", "value": 100 },
            "headers": {},
            "query": {}
        }
    }))?;
    harness.run_until_idle(100).await;

    Ok(())
}
