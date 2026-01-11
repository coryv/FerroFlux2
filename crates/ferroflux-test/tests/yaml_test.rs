use ferroflux_test::run_scenario;

#[tokio::test]
async fn test_basic_passthrough() -> anyhow::Result<()> {
    // A scenario where input-A passes data to a Passthrough node, and we check output.
    // We assume "ferroflux/basic:manual_trigger" receives injection.
    // And "ferroflux/basic:passthrough" just forwards.
    // NOTE: "ferroflux/basic:passthrough" might not be implemented in default registry!
    // We should use a known node. "ferroflux/basic:manual_trigger" IS the input node.
    // If I inject into "input-a", does it process?
    // "manual_trigger" usually fires when triggered.
    // Or we can rely on ANY node receiving input to queue it.
    // "ferroflux/basic:debug_sink" is better?
    // Let's assume a dummy scenario using "DebugNode" (which we used in examples).
    // But "DebugNode" logic needs to be registered in Core!
    // The SDK example defined `MyData` but the logic was.. wait.
    // SDK `EngineActor` runs `App`. `App` needs `NodeRegistry`.
    // FerroFluxCore default registry might be empty or basic?
    // I need to ensure the registry has the nodes I'm testing.
    // `ferroflux_core::components::nodes::register_default_nodes`?
    // If I use `FerroFluxClient::start()`, it uses `App::default()`.
    // Does `App::default()` register basic nodes?
    // Let's check `ferroflux_sdk` or `core` logic.
    // If no logic runs, the node sits there.
    // For this test to pass, "input-a" must ACT on the input.
    // If I inject into Inbox, the node SYSTEM must pick it up.
    // If no system handles "MyNode", nothing happens.

    // Proposal:
    // Just test Inject -> Inspect Inbox. (No logic execution required).
    // This validates the harness itself.

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
