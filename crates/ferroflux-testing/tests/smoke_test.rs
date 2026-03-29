use ferroflux_testing::harness::TestHarness;
use uuid::Uuid;
use serde_json::json;

#[tokio::test]
async fn test_harness_smoke() -> anyhow::Result<()> {
    let mut harness = TestHarness::new().await?;
    
    let node_id = Uuid::new_v4();
    
    let waml = format!(r#"
name: "Smoke Test"
nodes:
  - id: "{}"
    name: "Manual Node"
    type: "integration"
    config:
      integration: "core"
      action: "noop"
edges: []
"#, node_id);

    // 1. Load WAML
    harness.load_waml(&waml)?;
    
    // 2. Trigger the node
    harness.trigger_node(node_id, json!({}))?;
    
    // 3. Tick
    harness.run_until_idle(10).await;
    
    // 4. Verify some state?
    // For now, if it didn't panic and we can finish the test, that's a good smoke test.
    
    Ok(())
}
