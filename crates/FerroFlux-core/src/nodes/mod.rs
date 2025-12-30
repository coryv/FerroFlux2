use crate::traits::node_factory::NodeFactory;
use bevy_ecs::prelude::*;
use serde_json::Value;

// Macro to generate a simple factory for components that implement Deserialize
macro_rules! simple_factory {
    ($name:ident, $component:ty) => {
        pub struct $name;
        impl NodeFactory for $name {
            fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
                let c: $component = serde_json::from_value(config.clone())?;
                entity.insert(c);
                Ok(())
            }
            fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
                world
                    .get::<$component>(entity)
                    .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
            }
        }
    };
}

simple_factory!(HttpNodeFactory, crate::components::io::HttpConfig);
simple_factory!(WebhookNodeFactory, crate::components::io::WebhookConfig);
simple_factory!(CronNodeFactory, crate::components::io::CronConfig);
simple_factory!(SwitchNodeFactory, crate::components::logic::SwitchConfig);
simple_factory!(ScriptNodeFactory, crate::components::logic::ScriptConfig);
simple_factory!(
    SplitNodeFactory,
    crate::components::manipulation::SplitConfig
);
simple_factory!(
    TransformNodeFactory,
    crate::components::manipulation::TransformConfig
);
simple_factory!(
    StatsNodeFactory,
    crate::components::manipulation::StatsConfig
);
simple_factory!(
    WindowNodeFactory,
    crate::components::manipulation::WindowConfig
);
simple_factory!(
    ExpressionNodeFactory,
    crate::components::manipulation::ExpressionConfig
);
simple_factory!(
    CheckpointNodeFactory,
    crate::components::control::CheckpointConfig
);
simple_factory!(XmlNodeFactory, crate::components::connectors::XmlConfig);
simple_factory!(FtpNodeFactory, crate::components::connectors::FtpConfig);
simple_factory!(SshNodeFactory, crate::components::connectors::SshConfig);

pub struct AgentNodeFactory;
impl NodeFactory for AgentNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
        let config: crate::components::agent::AgentConfig = serde_json::from_value(config.clone())?;
        entity.insert(config);
        entity.insert(crate::components::schema::Requirements::default());
        entity.insert(crate::components::schema::ExpectedOutput::default());
        Ok(())
    }
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<crate::components::agent::AgentConfig>(entity)
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
    }
}

pub struct AggregateNodeFactory;
impl NodeFactory for AggregateNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
        let config: crate::components::manipulation::AggregateConfig =
            serde_json::from_value(config.clone())?;
        entity.insert(config);
        entity.insert(crate::components::manipulation::BatchState::default());
        Ok(())
    }
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<crate::components::manipulation::AggregateConfig>(entity)
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
    }
}

pub struct RssNodeFactory;
impl NodeFactory for RssNodeFactory {
    fn build(&self, entity: &mut EntityWorldMut, config: &Value) -> anyhow::Result<()> {
        let config: crate::components::connectors::RssConfig =
            serde_json::from_value(config.clone())?;
        entity.insert(config);
        entity.insert(crate::components::connectors::RssState::default());
        Ok(())
    }
    fn serialize(&self, world: &World, entity: Entity) -> Option<Value> {
        world
            .get::<crate::components::connectors::RssConfig>(entity)
            .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
    }
}

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
}

// System to register nodes
pub fn register_core_nodes(mut registry: ResMut<crate::resources::registry::NodeRegistry>) {
    registry.register("http", Box::new(HttpNodeFactory));
    registry.register("webhook", Box::new(WebhookNodeFactory));
    registry.register("agent", Box::new(AgentNodeFactory));
    registry.register("cron", Box::new(CronNodeFactory));
    registry.register("switch", Box::new(SwitchNodeFactory));
    registry.register("script", Box::new(ScriptNodeFactory));
    registry.register("integration", Box::new(IntegrationNodeFactory));
    registry.register("split", Box::new(SplitNodeFactory));
    registry.register("aggregate", Box::new(AggregateNodeFactory));
    registry.register("transform", Box::new(TransformNodeFactory));
    registry.register("stats", Box::new(StatsNodeFactory));
    registry.register("window", Box::new(WindowNodeFactory));
    registry.register("expression", Box::new(ExpressionNodeFactory));
    registry.register("checkpoint", Box::new(CheckpointNodeFactory));
    registry.register("rss", Box::new(RssNodeFactory));
    registry.register("xml", Box::new(XmlNodeFactory));
    registry.register("ftp", Box::new(FtpNodeFactory));
    registry.register("ssh", Box::new(SshNodeFactory));
}
