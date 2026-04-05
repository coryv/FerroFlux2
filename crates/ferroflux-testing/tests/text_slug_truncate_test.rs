use anyhow::Result;
use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_text_slug_truncate() -> Result<()> {
    // 1. Initialize Test Harness
    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;
    
    // Load the Text Slug/Truncate fixture
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/text_slug_truncate.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    harness.load_waml(&waml)?;

    // CASE 1: Full transformation pipeline
    // Input: "Hello World! This is a test."
    // Slug Result: "hello-world-this-is-a-test"
    // Truncate Result (10 chars): "hello-worl..."
    println!(">>> CASE 1: Slugify and truncate");
    
    let payload = json!({
        "body": "Hello World! This is a test."
    });

    let node_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(node_id, payload)?;

    harness.run_until_idle(100).await;

    Ok(())
}
