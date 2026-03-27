use ferroflux_sdk::testing::run_scenario;

#[tokio::test]
async fn test_basic_passthrough() -> anyhow::Result<()> {
    let yaml = r#"
name: "Harness Self-Test"
timeout_ms: 2000

blueprint:
  nodes:
    - id: "fe2e3fd0-fc95-428d-88f8-8460a32166f2"
      type: "core.action.log"
      name: "Test Node"
      config: {}
  edges: []

steps:
  - action: "inject"
    node: "fe2e3fd0-fc95-428d-88f8-8460a32166f2"
    value: { "foo": "bar" }

  - action: "wait"
    duration_ms: 500

  - action: "assert"
    node: "fe2e3fd0-fc95-428d-88f8-8460a32166f2"
    property: "outbox.last.context.foo.Inline"
    equals: "bar"
"#;

    run_scenario(yaml).await?;
    Ok(())
}
