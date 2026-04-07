use ferroflux_testing::harness::TestHarness;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_12_graphql_query() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    harness.load_platforms().ok();

    let mock_uri = harness.mock_server().uri();
    let gql_url = format!("{}/graphql", mock_uri);

    // Mock GraphQL Response
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "user": {
                    "name": "Jane GraphQL",
                    "email": "jane@gql.com"
                }
            }
        })))
        .mount(harness.mock_server())
        .await;

    // Load and trigger workflow
    let workflow_yaml = std::fs::read_to_string("fixtures/graphql_query.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"manual_trigger");
    
    // Trigger with variables and required event context
    harness.trigger_node(trigger_id, json!({
        "event": {
            "user_id": "U_TEST",
            "timestamp": 123456
        },
        "gql_url": gql_url,
        "variables": { "id": "U123" }
    }))?;

    harness.run_until_idle(100).await;

    // Verify mock request
    let requests = harness.mock_server().received_requests().await.unwrap();
    assert!(requests.len() >= 1, "expected at least 1 GraphQL request");

    Ok(())
}
