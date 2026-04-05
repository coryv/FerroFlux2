use anyhow::Result;
use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use ferroflux_core::resources::WorkDone;

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_6_math_pipeline() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    // 1. Initialize Test Harness
    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    harness.load_platforms()?;
    
    // Load the Case 3.6 fixture
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/case-3.6-math_pipeline.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    
    // Replace the placeholder URL with the full mock server URL
    let waml = waml.replace("/verify-total", &format!("{}/verify-total", mock_uri));
    harness.load_waml(&waml)?;

    // 2. Setup Mock Server for result verification
    Mock::given(method("POST"))
        .and(path("/verify-total"))
        .respond_with(ResponseTemplate::new(200))
        .mount(harness.mock_server())
        .await;

    // 3. Trigger the workflow
    let trigger_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    let payload = json!({
        "subtotal": 100.0,
        "tax_rate": 0.08
    });

    println!(">>> Triggering Case 3.6: Math Operations Pipeline");
    harness.trigger_node(trigger_uuid, payload)?;

    // 4. Run the engine manually to observe WorkDone
    println!(">>> Ticking engine manually...");
    for i in 0..50 {
        harness.tick();
        let wd = harness.app.world.resource::<WorkDone>().0;
        println!("Tick {}: WorkDone={}", i, wd);
        
        // If we received the HTTP call, we can stop early
        let received_requests = harness.mock_server().received_requests().await.unwrap_or_default();
        if received_requests.iter().any(|r| r.url.path() == "/verify-total") {
            println!(">>> Success! Received verify-total call at tick {}", i);
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    // 5. Final Verification
    let received_requests = harness.mock_server().received_requests().await
        .expect("Mock recording is disabled");
    
    let calls: Vec<_> = received_requests.into_iter()
        .filter(|r| r.url.path() == "/verify-total")
        .collect();

    assert!(!calls.is_empty(), "expected at least 1 HTTP call to /verify-total");
    
    let last_body: serde_json::Value = serde_json::from_slice(&calls[0].body)?;
    println!(">>> Received Body: {}", last_body);

    let result = last_body.as_f64().ok_or_else(|| anyhow::anyhow!("Result is not a number: {:?}", last_body))?;
    assert!((result - 108.0).abs() < 0.001, "Expected 108.0, got {}", result);

    println!(">>> Case 3.6: Math Operations Pipeline - PASSED");

    Ok(())
}
