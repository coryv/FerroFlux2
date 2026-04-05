use anyhow::Result;
use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_text_regex() -> Result<()> {
    // 1. Initialize Test Harness
    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;
    
    // Load the Text Regex fixture
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/text_regex.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    harness.load_waml(&waml)?;

    // CASE 1: Match and Replace
    // Input: "Contact us at support@example.com or sales@ferroflux.io"
    // Match Result: ["support@example.com", "sales@ferroflux.io"]
    // Replace Result: "Contact us at [REDACTED] or [REDACTED]"
    println!(">>> CASE 1: Regex match and replace");
    
    let payload = json!({
        "body": "Contact us at support@example.com or sales@ferroflux.io"
    });

    let node_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(node_id, payload)?;

    harness.run_until_idle(100).await;

    Ok(())
}
