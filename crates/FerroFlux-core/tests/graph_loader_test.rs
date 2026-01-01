use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use ferroflux_core::components::{
    core::{Edge, EdgeLabel, NodeConfig},
    pipeline::PipelineNode,
};
use ferroflux_core::domain::TenantId;
use ferroflux_core::graph_loader::load_graph_from_str;
use ferroflux_core::nodes::register_core_nodes;
use ferroflux_core::resources::NodeRouter;
use ferroflux_core::resources::registry::{DefinitionRegistry, NodeRegistry};

#[test]
fn test_graph_loading_basic() {
    let yaml = r#"
nodes:
  - id: "11111111-1111-1111-1111-111111111111"
    name: "My Agent"
    type: "core.action.agent"
    provider: "openai"
    model: "gpt-4o"
    system_instruction: "Sys"

  - id: "22222222-2222-2222-2222-222222222222"
    name: "My Switch"
    type: "core.action.switch"
    rules: []

  - id: "33333333-3333-3333-3333-333333333333"
    name: "My Http"
    type: "core.action.http"
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

    // Load YAML Definitions
    let mut def_registry = DefinitionRegistry::default();
    let platform_path = std::path::Path::new("../../platforms");
    if platform_path.exists() {
        def_registry
            .load_from_dir(platform_path)
            .expect("Failed to load platforms in test");
    }

    // Register nodes
    world.insert_resource(NodeRegistry::default());
    let mut system_state = SystemState::<ResMut<NodeRegistry>>::new(&mut world);
    let mut registry_res = system_state.get_mut(&mut world);

    // Register Legacy/Integration bridge
    register_core_nodes(&mut registry_res);

    // Register YAML nodes
    for (id, def) in &def_registry.definitions {
        registry_res.register(
            id,
            Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(
                def.clone(),
            )),
        );
    }

    let tenant_id = TenantId::from("default_tenant");
    let loader_result = load_graph_from_str(&mut world, tenant_id, yaml);
    assert!(
        loader_result.is_ok(),
        "Graph loading failed: {:?}",
        loader_result.err()
    );

    // Verify Nodes now use PipelineNode (the YAML-driven component)
    let mut query = world.query::<(&NodeConfig, &PipelineNode)>();
    let nodes: Vec<_> = query.iter(&world).collect();
    assert_eq!(nodes.len(), 3);

    // Verify Agent Node specifically
    let (node, pipeline) = nodes.iter().find(|(n, _)| n.name == "My Agent").unwrap();
    assert_eq!(node.id.to_string(), "11111111-1111-1111-1111-111111111111");
    assert_eq!(pipeline.definition_id, "core.action.agent");

    // Verify Edges
    let mut edge_query = world.query::<(&Edge, Option<&EdgeLabel>)>();
    let edges: Vec<_> = edge_query.iter(&world).collect();
    assert_eq!(edges.len(), 2);

    // Verify Node Router
    let router = world.resource::<NodeRouter>();
    assert_eq!(router.0.len(), 3);
}
