use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;
use uuid::Uuid;
use ferroflux_core::resources::WorkDone;

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_8_crypto_hash_uuid() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    harness.load_platforms()?;

    // 2. Setup Mock Server for result verification
    Mock::given(method("POST"))
        .and(path("/verify-crypto"))
        .respond_with(ResponseTemplate::new(200))
        .mount(harness.mock_server())
        .await;

    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/case-3.8-crypto_hash_uuid.yaml");
    let waml = std::fs::read_to_string(fixture_path)?;
    
    // Replace URL in WAML
    let waml = waml.replace("/verify-crypto", &format!("{}/verify-crypto", mock_uri));
    harness.load_waml(&waml)?;

    // 3. Trigger the workflow
    let trigger_uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"webhook_in");
    let payload = json!({
        "event": {
            "body": {
                "input": "hello world"
            },
            "headers": {},
            "query": {}
        }
    });

    println!(">>> Triggering Case 3.8: Crypto Hash and UUID");
    harness.trigger_node(trigger_uuid, payload)?;

    // 4. Run the engine
    println!(">>> Ticking engine manually...");
    for i in 0..50 {
        harness.tick();
        let wd = harness.app.world.resource::<WorkDone>().0;
        println!("Tick {}: WorkDone={}", i, wd);
        
        let requests = harness.mock_server().received_requests().await.unwrap_or_default();
        if requests.iter().any(|r| r.url.path() == "/verify-crypto") {
            println!(">>> Success! Received verify-crypto call at tick {}", i);
            break;
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    // 5. Final Verification
    let received_requests = harness.mock_server().received_requests().await.expect("mock recording failed");
    let req = received_requests.iter()
        .find(|r| r.url.path() == "/verify-crypto")
        .expect("should have called /verify-crypto");
    
    let body: serde_json::Value = serde_json::from_slice(&req.body)?;
    println!(">>> Received Body: {}", body);

    // Should be a UUID string
    let uuid_str = body.as_str().expect("Result is not a string");
    assert!(Uuid::parse_str(uuid_str).is_ok(), "Result is not a valid UUID: {}", uuid_str);

    println!(">>> Case 3.8: Crypto Hash and UUID - PASSED");

    Ok(())
}
