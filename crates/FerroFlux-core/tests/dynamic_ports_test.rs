use ferroflux_core::nodes::definition::{
    Interface, NodeDefinition, NodeMeta, PortDef, PortGeneratorDef, PortTemplateDef,
};
use ferroflux_core::nodes::yaml_factory::YamlNodeFactory;
use ferroflux_core::traits::node_factory::NodeFactory;
use serde_json::json;

#[test]
fn test_dynamic_port_generation() {
    // 1. Setup Definition
    let def = NodeDefinition {
        meta: NodeMeta {
            id: "test.dynamic".into(),
            name: "Dynamic Node".into(),
            category: "Test".into(),
            node_type: "Action".into(),
            description: None,
            version: None,
            platform: None,
            data_strategy: None,
            node_subtype: None,
            signature: None,
            permissions: vec![],
        },
        interface: Interface {
            inputs: vec![],
            outputs: vec![
                // Static Port
                PortDef {
                    name: "default".into(),
                    data_type: "flow".into(),
                    default_hidden: false,
                    generator: None,
                },
                // Dynamic Generator
                PortDef {
                    name: "".into(), // Empty name for pure generator
                    data_type: "".into(),
                    default_hidden: false,
                    generator: Some(PortGeneratorDef {
                        source: "settings.rules".into(),
                        template: PortTemplateDef {
                            name: "{{ item.label }}".into(),
                            data_type: "flow".into(),
                        },
                    }),
                },
            ],
            settings: vec![],
        },
        context: None,
        execution: vec![],
        output_transform: None,
        routing: None,
    };

    let factory = YamlNodeFactory::new(def);

    // 2. Setup Config
    let config = json!({
        "rules": [
            { "label": "Case A", "condition": "x > 1" },
            { "label": "Case B", "condition": "x < 1" }
        ]
    });

    // 3. Resolve
    let (inputs, outputs) = factory.resolve_interface(&config);

    // 4. Verify
    assert_eq!(inputs.len(), 0);
    assert_eq!(outputs.len(), 3); // 1 static + 2 dynamic

    assert_eq!(outputs[0].name, "default");
    assert_eq!(outputs[1].name, "Case A");
    assert_eq!(outputs[2].name, "Case B");
}
