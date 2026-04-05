use anyhow::Result;
use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use ferroflux_core::resources::WorkDone;

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_7_date_parse_diff() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    // 1. Initialize Test Harness
    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    harness.load_platforms()?;
    
    // Load the Case 3.7 fixture
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/case-3.7-date_parse_diff.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    
    // Replace the placeholder URL with the full mock server URL
    let waml = waml.replace("/verify-diff", &format!("{}/verify-diff", mock_uri));
    harness.load_waml(&waml)?;

    // 2. Setup Mock Server for result verification
    Mock::given(method("POST"))
        .and(path("/verify-diff"))
        .respond_with(ResponseTemplate::new(200))
        .mount(harness.mock_server())
        .await;

    // 3. Trigger the workflow
    let trigger_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    let payload = json!({
        "event": {
            "body": {
                "start_date": "2023-01-01T00:00:00Z",
                "end_date": "2023-01-05T12:00:00Z"
            },
            "headers": {},
            "query": {}
        }
    });

    println!(">>> Triggering Case 3.7: Date Parse and Diff");
    harness.trigger_node(trigger_uuid, payload)?;

    // 4. Run the engine manually to observe WorkDone
    println!(">>> Ticking engine manually...");
    for i in 0..50 {
        harness.tick();
        let wd = harness.app.world.resource::<WorkDone>().0;
        println!("Tick {}: WorkDone={}", i, wd);
        
        // If we received the HTTP call, we can stop early
        let received_requests = harness.mock_server().received_requests().await.unwrap_or_default();
        if received_requests.iter().any(|r| r.url.path() == "/verify-diff") {
            println!(">>> Success! Received verify-diff call at tick {}", i);
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    // 5. Final Verification
    let received_requests = harness.mock_server().received_requests().await
        .expect("Mock recording is disabled");
    
    let calls: Vec<_> = received_requests.into_iter()
        .filter(|r| r.url.path() == "/verify-diff")
        .collect();

    assert!(!calls.is_empty(), "expected at least 1 HTTP call to /verify-diff");
    
    let last_body: serde_json::Value = serde_json::from_slice(&calls[0].body)?;
    println!(">>> Received Body: {}", last_body);

    let days = last_body["days"].as_f64().expect("Expected days to be a number");
    assert_eq!(days, 4.0, "Expected 4.0 days passed, got {}", days);

    println!(">>> Case 3.7: Date Parse and Diff - PASSED");

    Ok(())
}
