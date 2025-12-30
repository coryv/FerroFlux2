use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use ferroflux_core::components::{
    agent::AgentConfig,
    core::{Edge, EdgeLabel, NodeConfig},
    io::HttpConfig,
    logic::SwitchConfig,
};
use ferroflux_core::domain::TenantId;
use ferroflux_core::graph_loader::load_graph_from_str;
use ferroflux_core::nodes::register_core_nodes;
use ferroflux_core::resources::NodeRouter;
use ferroflux_core::resources::registry::NodeRegistry;

#[test]
fn test_graph_loading_basic() {
    let yaml = r#"
nodes:
  - id: "11111111-1111-1111-1111-111111111111"
    name: "My Agent"
    type: "Agent"
    provider: "openai"
    model: "gpt-4o"
    system_instruction: "Sys"
    user_prompt_template: "User"
    result_key: "agent_out"

  - id: "22222222-2222-2222-2222-222222222222"
    name: "My Switch"
    type: "Switch"
    script: "input.val > 10"

  - id: "33333333-3333-3333-3333-333333333333"
    name: "My Http"
    type: "Http"
    url: "http://example.com"
    method: "POST"

edges:
  - source_id: "11111111-1111-1111-1111-111111111111"
    target_id: "22222222-2222-2222-2222-222222222222"
  - source_id: "22222222-2222-2222-2222-222222222222"
    target_id: "33333333-3333-3333-3333-333333333333"
    label: "true"
"#;

    let mut world = World::new();
    world.insert_resource(NodeRouter::default());

    // Register nodes
    world.insert_resource(NodeRegistry::default());
    let mut system_state = SystemState::<ResMut<NodeRegistry>>::new(&mut world);
    let registry_res = system_state.get_mut(&mut world);
    register_core_nodes(registry_res);

    let tenant_id = TenantId::from("default_tenant");
    let loader_result = load_graph_from_str(&mut world, tenant_id, yaml);
    assert!(
        loader_result.is_ok(),
        "Graph loading failed: {:?}",
        loader_result.err()
    );

    // Verify Agent Node
    let mut agent_query = world.query::<(&NodeConfig, &AgentConfig)>();
    let (node, agent) = agent_query
        .iter(&world)
        .find(|(n, _)| n.name == "My Agent")
        .expect("Agent node not found");

    assert_eq!(node.id.to_string(), "11111111-1111-1111-1111-111111111111");
    assert_eq!(agent.provider, "openai");
    assert_eq!(agent.model, "gpt-4o");
    assert_eq!(agent.result_key, Some("agent_out".to_string()));

    // Verify Switch Node
    let mut switch_query = world.query::<(&NodeConfig, &SwitchConfig)>();
    let (_, switch) = switch_query
        .iter(&world)
        .find(|(n, _)| n.name == "My Switch")
        .expect("Switch node not found");
    assert_eq!(switch.script, "input.val > 10");

    // Verify Http Node
    let mut http_query = world.query::<(&NodeConfig, &HttpConfig)>();
    let (_, http) = http_query
        .iter(&world)
        .find(|(n, _)| n.name == "My Http")
        .expect("Http node not found");
    assert_eq!(http.url, "http://example.com");
    assert_eq!(http.method, "POST");

    // Verify Edges
    let mut edge_query = world.query::<(&Edge, Option<&EdgeLabel>)>();
    let edges: Vec<_> = edge_query.iter(&world).collect();
    assert_eq!(edges.len(), 2);

    // Verify Node Router
    let router = world.resource::<NodeRouter>();
    assert_eq!(router.0.len(), 3);
}
