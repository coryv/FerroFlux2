use crate::traits::node_factory::{NodeFactory, NodeMetadata};
use bevy_ecs::prelude::*;
use serde_json::Value;

pub mod definition;
pub mod yaml_factory;

pub struct IntegrationNodeFactory;
impl NodeFactory for IntegrationNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
        let c: crate::components::IntegrationConfig = serde_json::from_value(config.clone())?;

        let registry_opt = entity
            .world()
            .get_resource::<crate::integrations::IntegrationRegistry>()
            .cloned();

        if let Some(registry) = registry_opt {
            let def = registry.definitions.get(&c.integration).ok_or_else(|| {
                anyhow::anyhow!("Integration '{}' not found in registry", c.integration)
            })?;

            let action_def = def.actions.get(&c.action).ok_or_else(|| {
                anyhow::anyhow!(
                    "Action '{}' not found in integration '{}'",
                    c.action,
                    c.integration
                )
            })?;

            // Construct derived components
            let action_config = &action_def.implementation.config;
            let full_url = format!("{}{}", def.base_url, action_config.path);

            let http_config = crate::components::io::HttpConfig {
                url: full_url,
                method: action_config.method.clone(),
                result_key: None,
                connection_slug: None,
            };

            let requirements = crate::components::schema::Requirements {
                needed_fields: action_def.inputs.iter().map(|i| i.name.clone()).collect(),
            };

            let expected_output = crate::components::schema::ExpectedOutput {
                aggregated_schema: action_def.outputs.iter().map(|o| o.name.clone()).collect(),
            };

            // Insert the original config AND the compiled HttpConfig
            entity.insert(c);
            entity.insert(http_config);
            entity.insert(requirements);
            entity.insert(expected_output);
            entity.insert(crate::components::Inbox::default());
            entity.insert(crate::components::Outbox::default());

            // Hydrate PayloadMapper (Always insert if headers or template exist)
            if action_config.body_template.is_some() || !action_config.headers.is_empty() {
                entity.insert(crate::components::integration::PayloadMapper {
                    template: action_config.body_template.clone(),
                    headers: action_config.headers.clone(),
                });
            }

            // Hydrate AuthConfig
            if let Some(auth_def) = &def.auth {
                use crate::integrations::registry::AuthDef;
                match auth_def {
                    AuthDef::Basic => {
                        entity.insert(crate::components::AuthConfig::Basic {
                            user_env: format!("{}_USER", def.name.to_uppercase()),
                            pass_env: format!("{}_PASS", def.name.to_uppercase()),
                        });
                    }
                    AuthDef::ApiKey {
                        in_header,
                        key_name,
                    } => {
                        let env_var = format!("{}_API_KEY", def.name.to_uppercase());
                        if *in_header {
                            entity.insert(crate::components::AuthConfig::ApiKey {
                                key_env: env_var,
                                header: Some(key_name.clone()),
                                query: None,
                            });
                        } else {
                            entity.insert(crate::components::AuthConfig::ApiKey {
                                key_env: env_var,
                                header: None,
                                query: Some(key_name.clone()),
                            });
                        }
                    }
                    AuthDef::OAuth2 { .. } | AuthDef::Bearer => {
                        entity.insert(crate::components::AuthConfig::Bearer {
                            token_env: format!("{}_TOKEN", def.name.to_uppercase()),
                        });
                    }
                }
            }
        }

        Ok(())
    }
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<crate::components::IntegrationConfig>(entity)
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
    }
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "integration".to_string(),
            name: "Integration".to_string(),
            category: "Internal".to_string(),
            platform: None,
            description: None,
            inputs: vec![],
            outputs: vec![],
            settings: vec![],
        }
    }
}

// System to register nodes (can also be called manually)
pub fn register_core_nodes(registry: &mut crate::resources::registry::NodeRegistry) {
    println!("DEBUG: registering core nodes");
    // We only register the Integration bridge for now.
    // All other core nodes are loaded via YAML from the platforms/ directory.
    registry.register("integration", Box::new(IntegrationNodeFactory));
}
