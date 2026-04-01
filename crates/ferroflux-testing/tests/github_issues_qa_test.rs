use ferroflux_testing::harness::TestHarness;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn test_github_issues_create() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();

    // NOTE: load_platforms() fails due to a pre-existing YAML parse error in
    // platforms/core/utils.graphql.yaml (malformed block scalar at line 68).
    // The error aborts the entire directory load, so 'github' is never registered.
    // We work around this by loading the github platform directly via add_mocked_integration.
    let github_platform_yaml = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../platforms/github/github.yaml")
    )?;
    harness.add_mocked_integration(&github_platform_yaml)?;

    // Also load the github.issues.create node definition explicitly
    harness.load_platforms().ok(); // best-effort for core nodes; errors are expected

    // GitHub issues.create makes POST /repos/{owner}/{repo}/issues — expect 201
    Mock::given(method("POST"))
        .and(path("/repos/acme/app/issues"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 1,
            "number": 42,
            "title": "Bug found",
            "body": "Details here",
            "html_url": "https://github.com/acme/app/issues/42",
            "state": "open"
        })))
        .mount(harness.mock_server())
        .await;

    let workflow_yaml = std::fs::read_to_string("fixtures/github_issues_qa.yaml")?;
    harness.load_waml(&workflow_yaml)?;

    let trigger_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(trigger_uuid, json!({
        "event": {
            "body": {
                "title": "Bug found",
                "body": "Details here",
                "owner": "acme",
                "repo": "app"
            },
            "headers": {},
            "query": {}
        }
    }))?;

    harness.run_until_idle(100).await;

    let requests = harness.mock_server()
        .received_requests().await
        .expect("mock server should record requests");

    // Assertion 1: at least one HTTP call was made
    assert!(
        requests.len() >= 1,
        "expected at least 1 HTTP call to GitHub API, got 0"
    );

    // Assertion 2: POST /repos/acme/app/issues was called
    let create_req = requests.iter()
        .find(|r| r.url.path() == "/repos/acme/app/issues")
        .expect("engine should have called POST /repos/acme/app/issues");

    // Assertion 3: request body contains correct title
    let body: serde_json::Value = serde_json::from_slice(&create_req.body)?;
    assert_eq!(
        body["title"], "Bug found",
        "issue title should be 'Bug found', got {:?}",
        body["title"]
    );

    // Assertion 4: request body contains correct body text
    assert_eq!(
        body["body"], "Details here",
        "issue body should be 'Details here', got {:?}",
        body["body"]
    );

    println!("All assertions passed — POST /repos/acme/app/issues received with correct payload.");
    Ok(())
}
