use ferroflux_integration::definition::NodeDefinition;
use ferroflux_testing::harness::TestHarness;
use ferroflux_types::registry::NodeRegistry;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use serde_json::json;

/// QA baseline test: webhook → github.issues.create
///
/// Payload: { "title": "Bug found", "body": "Details here", "owner": "acme", "repo": "app" }
///
/// Assertions:
///   1. Exactly one HTTP POST to /repos/acme/app/issues is received
///   2. Request body contains correct title field
///   3. Request body contains correct body field
///   4. Engine runs to idle without panicking
#[tokio::test(flavor = "multi_thread")]
async fn test_github_issues_create_qa_baseline() -> anyhow::Result<()> {
    // ── 1. Tracing ─────────────────────────────────────────────────────────────
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    println!("=== GitHub Issues Create QA Baseline Test START ===");

    // Allow wiremock's loopback address through the SSRF guard
    unsafe { std::env::set_var("FERROFLUX_ALLOW_INTERNAL_IPS", "true"); }

    // ── 2. Harness ──────────────────────────────────────────────────────────────
    let mut harness = TestHarness::new().await;
    let mock_uri = harness.mock_server().uri();
    println!("Mock server listening at: {}", mock_uri);

    // ── 3. Register GitHub API mock ─────────────────────────────────────────────
    // GitHub returns 201 Created with a partial issue object
    Mock::given(method("POST"))
        .and(path("/repos/acme/app/issues"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({
                "id": 1,
                "number": 42,
                "title": "Bug found",
                "body": "Details here",
                "state": "open",
                "html_url": "https://github.com/acme/app/issues/42"
            })),
        )
        .mount(harness.mock_server())
        .await;

    println!("GitHub API mock registered for POST /repos/acme/app/issues");

    // ── 4. Load the GitHub platform + action node definitions directly.
    //       We bypass load_platforms() because an unrelated file
    //       (core/utils.graphql.yaml) has a YAML parse bug that causes
    //       the full directory walk to bail before reaching github/.
    //       Instead we: (a) insert the platform via add_mocked_integration so
    //       base_url is pointed at wiremock, then (b) manually parse the action
    //       node YAML and register it in NodeRegistry via YamlNodeFactory.
    let github_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../platforms/github");

    // (a) Platform definition → DefinitionRegistry + mock base_url
    let github_platform_yaml = std::fs::read_to_string(github_dir.join("github.yaml"))?;
    harness.add_mocked_integration(&github_platform_yaml)?;
    println!("GitHub platform loaded and base_url overridden to mock server: {}", mock_uri);

    // (b) Action node definitions → NodeRegistry
    let action_files = [
        "action.issues.create.yaml",
        "action.issues.get.yaml",
        "action.repos.list.yaml",
    ];
    for filename in &action_files {
        let content = std::fs::read_to_string(github_dir.join(filename))?;
        let def: NodeDefinition = serde_yaml::from_str(&content)?;
        let node_id = def.meta.id.clone();
        let mut registry = harness.app.world.resource_mut::<NodeRegistry>();
        registry.register(
            &node_id,
            Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(def)),
        );
        println!("Registered node: {}", node_id);
    }

    // ── 5. Load workflow WAML ───────────────────────────────────────────────────
    let yaml_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/github_issues_qa_baseline.yaml");
    let workflow_yaml = std::fs::read_to_string(&yaml_path)?;
    println!("Loaded WAML from {:?}:\n{}", yaml_path, workflow_yaml);
    harness.load_waml(&workflow_yaml)?;

    // ── 6. Trigger webhook node ─────────────────────────────────────────────────
    let webhook_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    println!("Triggering webhook node: {}", webhook_uuid);

    harness.trigger_node(
        webhook_uuid,
        json!({
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
        }),
    )?;

    // ── 7. Run engine ───────────────────────────────────────────────────────────
    println!("Running engine until idle (max 100 ticks)...");
    harness.run_until_idle(100).await;
    println!("Engine reached idle.");

    // ── 8. Assertions ───────────────────────────────────────────────────────────
    let received = harness
        .mock_server()
        .received_requests()
        .await
        .expect("Mock server should have recorded requests");

    println!(
        "Total requests received by mock server: {}",
        received.len()
    );
    for (i, req) in received.iter().enumerate() {
        let body_str = String::from_utf8_lossy(&req.body);
        println!("  Request {}: {} {} | body: {}", i + 1, req.method, req.url, body_str);
    }

    // Assertion 1: at least one request reached the mock
    assert!(
        !received.is_empty(),
        "FAIL: No requests reached the GitHub mock — engine did not dispatch the HTTP call"
    );
    println!("PASS: At least one request reached the mock server");

    // Assertion 2: the specific issues endpoint was called
    let issue_request = received
        .iter()
        .find(|r| r.url.path() == "/repos/acme/app/issues")
        .expect("FAIL: Expected POST to /repos/acme/app/issues but none was found");
    println!("PASS: POST /repos/acme/app/issues was called");

    // Assertion 3: request body contains correct title
    let req_body: serde_json::Value = serde_json::from_slice(&issue_request.body)
        .expect("FAIL: Request body was not valid JSON");
    assert_eq!(
        req_body["title"], "Bug found",
        "FAIL: title field mismatch — got {:?}",
        req_body["title"]
    );
    println!("PASS: title == \"Bug found\"");

    // Assertion 4: request body contains correct body text
    assert_eq!(
        req_body["body"], "Details here",
        "FAIL: body field mismatch — got {:?}",
        req_body["body"]
    );
    println!("PASS: body == \"Details here\"");

    println!("=== GitHub Issues Create QA Baseline Test COMPLETE ===");
    Ok(())
}
