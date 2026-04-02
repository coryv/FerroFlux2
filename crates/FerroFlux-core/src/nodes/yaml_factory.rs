use crate::components::pipeline::PipelineNode;
use crate::nodes::definition::NodeDefinition;
use crate::traits::node_factory::{NodeFactory, NodeMetadata, PortMetadata};
use anyhow::Result;
use bevy_ecs::prelude::*;
use serde_json::Value;

/// A Generic Node Factory that spawns entities based on a YAML definition.
pub struct YamlNodeFactory {
    pub definition: NodeDefinition,
}

impl YamlNodeFactory {
    pub fn new(definition: NodeDefinition) -> Self {
        Self { definition }
    }
}

impl NodeFactory for YamlNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> Result<()> {
        // Create the runtime component
        // Current config is merged with defaults in the pipeline system,
        // but here we just persist the instance config.
        let config_map: std::collections::HashMap<String, Value> =
            serde_json::from_value(config.clone()).unwrap_or_default();

        let node = PipelineNode::new(self.definition.meta.id.clone(), config_map);

        entity.insert(node);
        entity.insert(crate::components::Inbox::default());
        entity.insert(crate::components::Outbox::default());
        Ok(())
    }

    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<PipelineNode>(entity)
            .map(|n| serde_json::to_value(&n.config).unwrap_or(Value::Null))
    }

    fn metadata(&self) -> NodeMetadata {
        let meta = &self.definition.meta;
        let interface = &self.definition.interface;

        NodeMetadata {
            id: meta.id.clone(),
            name: meta.name.clone(),
            category: meta.category.clone(),
            platform: meta.platform.clone(),
            description: meta.description.clone(),
            inputs: interface
                .inputs
                .iter()
                .map(|p| PortMetadata {
                    name: p.name.clone(),
                    data_type: p.data_type.clone(),
                })
                .collect(),
            outputs: interface
                .outputs
                .iter()
                .map(|p| PortMetadata {
                    name: p.name.clone(),
                    data_type: p.data_type.clone(),
                })
                .collect(),
            settings: interface
                .settings
                .iter()
                .map(|s| serde_json::to_value(s).unwrap())
                .collect(),
            discovery: meta.discovery.as_ref().map(|d| ferroflux_types::node_factory::DiscoveryMetadata {
                purpose: d.purpose.clone(),
                usage_context: d.usage_context.clone(),
                agent_hints: d.agent_hints.clone(),
                waml_examples: d.waml_examples.iter().map(|e| ferroflux_types::node_factory::WamlExample {
                    title: e.title.clone(),
                    waml: e.waml.clone(),
                }).collect(),
            }),
        }
    }

    fn resolve_interface(&self, config: &Value) -> (Vec<PortMetadata>, Vec<PortMetadata>) {
        let interface = &self.definition.interface;

        // Helper to expand ports
        let expand_ports = |ports: &[crate::nodes::definition::PortDef]| -> Vec<PortMetadata> {
            let mut result = Vec::new();
            for p in ports {
                // 1. If static name exists, add it (unless it's purely a generator container?)
                // The proposal allowed both. If name is present, include it?
                // Usually generator is exclusive or supplementary.
                // Let's assume if 'name' is non-empty, it's a static port.
                if !p.name.is_empty() {
                    result.push(PortMetadata {
                        name: p.name.clone(),
                        data_type: p.data_type.clone(),
                    });
                }

                // 2. Generator expansion
                if let Some(generator_def) = &p.generator {
                    // source e.g. "settings.rules" -> we expect config to have { "rules": [...] }
                    // If source starts with "settings.", strip it because 'config' IS the settings object.
                    let source_path = if generator_def.source.starts_with("settings.") {
                        &generator_def.source["settings.".len()..]
                    } else {
                        &generator_def.source
                    };

                    // Simple path resolution (supports nested like "foo.bar" via pointer?)
                    // Config is usually flat or simple nested. Use pointer if starts with /?
                    // Or just use pointer notation always?
                    // Let's try to convert dot notation to pointer if needed, or just specific logic.
                    // For now, assume strict mapping or slash pointer.
                    // Actually, let's treat source as a pointer directly if it starts with /, else look at root.
                    // But standard is "settings.rules".
                    // Let's implement simple dot walking or just top-level check for MVP.

                    let mut current_val = config;
                    for part in source_path.split('.') {
                        if let Some(v) = current_val.get(part) {
                            current_val = v;
                        } else {
                            current_val = &Value::Null;
                            break;
                        }
                    }

                    if let Some(arr) = current_val.as_array() {
                        for item in arr {
                            // Render name template
                            // Context: { "item": item, "config": config }
                            let ctx = serde_json::json!({
                                "item": item,
                                "config": config
                            });

                            let name = crate::systems::io::templating::apply_template(
                                &generator_def.template.name,
                                &ctx,
                            );

                            result.push(PortMetadata {
                                name,
                                data_type: generator_def.template.data_type.clone(),
                            });
                        }
                    }
                }
            }
            result
        };

        (
            expand_ports(&interface.inputs),
            expand_ports(&interface.outputs),
        )
    }
}
