use anyhow::Result;
use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_json_merge() -> Result<()> {
    // 1. Initialize Test Harness
    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;
    
    // Load the JSON Merge fixture
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/json_merge.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    harness.load_waml(&waml)?;

    // CASE 1: Deep recursive merge
    // Base: {"a": 1, "b": {"c": 2}}
    // Other: {"b": {"d": 3}, "e": 4}
    // Result: {"a": 1, "b": {"c": 2, "d": 3}, "e": 4}
    println!(">>> CASE 1: Deep recursive merge");
    
    // In our fixture, we map 'body' to 'input' and 'other' to 'other'.
    // The webhook trigger (1v1) outputs both if we provide them in the payload.
    // Wait! Let's check how webhook trigger handles payloads.
    // Standard trigger logic: it takes the whole payload as 'body'.
    // BUT we added an edge for 'other'. This requires 'other' to be a top-level key in the trigger's output.
    
    let payload = json!({
        "body": {
            "a": 1,
            "b": { "c": 2 }
        },
        "other": {
            "b": { "d": 3 },
            "e": 4
        }
    });

    let node_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(node_id, payload)?;

    harness.run_until_idle(100).await;

    // CASE 2: Merge with Null Overrides (Delete pattern)
    // Base: {"a": 1, "b": 2}
    // Other: {"a": null, "c": 3}
    // Result: {"a": null, "b": 2, "c": 3}
    println!("\n>>> CASE 2: Merge with Null overrides");
    
    let payload2 = json!({
        "body": {
            "a": 1,
            "b": 2
        },
        "other": {
            "a": null,
            "c": 3
        }
    });

    harness.trigger_node(node_id, payload2)?;

    harness.run_until_idle(100).await;

    Ok(())
}
