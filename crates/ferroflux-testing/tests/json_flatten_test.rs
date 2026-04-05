use anyhow::Result;
use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_json_flatten() -> Result<()> {
    // 1. Initialize Test Harness
    let mut harness = TestHarness::new().await;
    harness.load_platforms()?;
    
    // Load the JSON Flatten fixture
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/json_flatten.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    harness.load_waml(&waml)?;

    // CASE 1: Deep dot-notation flattening
    // Input: {"a": {"b": {"c": 1}}, "d": [10, 20]}
    // Result: {"a.b.c": 1, "d.0": 10, "d.1": 20}
    println!(">>> CASE 1: Deep dot-notation flattening");
    
    let payload = json!({
        "body": {
            "user": {
                "id": 123,
                "profile": {
                    "firstName": "John",
                    "lastName": "Doe"
                },
                "roles": ["admin", "editor"]
            }
        }
    });

    let node_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(node_id, payload)?;

    harness.run_until_idle(100).await;

    // CASE 2: Flat object remains flat
    // Input: {"a": 1, "b": 2}
    // Result: {"a": 1, "b": 2}
    println!("\n>>> CASE 2: Flat object remains flat");
    
    let payload2 = json!({
        "body": {
            "status": "success",
            "code": 200
        }
    });

    harness.trigger_node(node_id, payload2)?;

    harness.run_until_idle(100).await;

    Ok(())
}
