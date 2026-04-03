use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_json_query() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;

    let waml = std::fs::read_to_string("fixtures/json_query.yaml")?;
    harness.load_waml(&waml)?;

    // CASE 1: Simple Path Extraction
    // Input: {"user": {"profile": {"name": "Alice"}}}
    // Query: "user.profile.name"
    println!(">>> CASE 1: Simple path extraction");
    harness.set_workflow_config("query_path", json!("'user.profile.name'"))?;

    let payload = json!({
        "user": {
            "profile": {
                "name": "Alice"
            }
        }
    });

    let node_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(node_id, payload)?;

    harness.run_until_idle(100).await;

    // CASE 2: Array Filtering
    // Input: {"orders": [{"id": 1, "status": "shipped"}, {"id": 2, "status": "pending"}]}
    // Query: "orders[?status == 'pending'].id | [0]"
    println!("\n>>> CASE 2: Array filtering");
    // We need to re-load or update the config. Harness set_workflow_config should work.
    harness.set_workflow_config("query_path", json!("'orders[?status == \\'pending\\'].id | [0]'"))?;

    let payload2 = json!({
        "orders": [
            { "id": 1, "status": "shipped" },
            { "id": 2, "status": "pending" }
        ]
    });

    harness.trigger_node(node_id, payload2)?;

    harness.run_until_idle(100).await;

    Ok(())
}
