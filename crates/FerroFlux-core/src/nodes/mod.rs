use crate::traits::node_factory::{NodeFactory, NodeMetadata, PortMetadata};
use bevy_ecs::prelude::*;
use serde_json::Value;

pub mod definition;
pub mod yaml_factory;

// Macro to generate a simple factory for components that implement Deserialize
macro_rules! simple_factory {
    ($name:ident, $component:ty, $meta:expr) => {
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
            fn metadata(&self) -> NodeMetadata {
                $meta
            }
        }
    };
}

simple_factory!(
    HttpNodeFactory,
    crate::components::io::HttpConfig,
    NodeMetadata {
        id: "http".to_string(),
        name: "HTTP Request".to_string(),
        category: "Services".to_string(),
        description: Some("Make HTTP requests".to_string()),
        inputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "URL".to_string(),
                data_type: "string".to_string()
            }
        ],
        outputs: vec![
            PortMetadata {
                name: "Success".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Error".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Response".to_string(),
                data_type: "json".to_string()
            }
        ]
    }
);

simple_factory!(
    WebhookNodeFactory,
    crate::components::io::WebhookConfig,
    NodeMetadata {
        id: "webhook".to_string(),
        name: "Webhook".to_string(),
        category: "Events".to_string(),
        description: Some("Receive external webhooks".to_string()),
        inputs: vec![],
        outputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Body".to_string(),
                data_type: "json".to_string()
            }
        ]
    }
);

simple_factory!(
    CronNodeFactory,
    crate::components::io::CronConfig,
    NodeMetadata {
        id: "cron".to_string(),
        name: "Cron Schedule".to_string(),
        category: "Events".to_string(),
        description: Some("Trigger on schedule".to_string()),
        inputs: vec![],
        outputs: vec![PortMetadata {
            name: "Exec".to_string(),
            data_type: "flow".to_string()
        }]
    }
);

simple_factory!(
    SwitchNodeFactory,
    crate::components::logic::SwitchConfig,
    NodeMetadata {
        id: "switch".to_string(),
        name: "Switch / If".to_string(),
        category: "Logic".to_string(),
        description: Some("Branch based on condition".to_string()),
        inputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Condition".to_string(),
                data_type: "boolean".to_string()
            }
        ],
        outputs: vec![
            PortMetadata {
                name: "True".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "False".to_string(),
                data_type: "flow".to_string()
            }
        ]
    }
);

simple_factory!(
    ScriptNodeFactory,
    crate::components::logic::ScriptConfig,
    NodeMetadata {
        id: "script".to_string(),
        name: "Rhai Script".to_string(),
        category: "Logic".to_string(),
        description: Some("Run custom Rhai script".to_string()),
        inputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Input".to_string(),
                data_type: "any".to_string()
            }
        ],
        outputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Result".to_string(),
                data_type: "any".to_string()
            }
        ]
    }
);

simple_factory!(
    SplitNodeFactory,
    crate::components::manipulation::SplitConfig,
    NodeMetadata {
        id: "split".to_string(),
        name: "Split Array".to_string(),
        category: "Transform".to_string(),
        description: Some("Split array into individual items".to_string()),
        inputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Array".to_string(),
                data_type: "array".to_string()
            }
        ],
        outputs: vec![PortMetadata {
            name: "Item".to_string(),
            data_type: "flow".to_string()
        }]
    }
);

simple_factory!(
    TransformNodeFactory,
    crate::components::manipulation::TransformConfig,
    NodeMetadata {
        id: "transform".to_string(),
        name: "Transform JSON".to_string(),
        category: "Transform".to_string(),
        description: Some("Map/Transform JSON structure".to_string()),
        inputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Input".to_string(),
                data_type: "json".to_string()
            }
        ],
        outputs: vec![
            PortMetadata {
                name: "Exec".to_string(),
                data_type: "flow".to_string()
            },
            PortMetadata {
                name: "Output".to_string(),
                data_type: "json".to_string()
            }
        ]
    }
);

simple_factory!(
    StatsNodeFactory,
    crate::components::manipulation::StatsConfig,
    NodeMetadata {
        id: "stats".to_string(),
        name: "Statistics".to_string(),
        category: "Math".to_string(),
        description: Some("Calculate stats on stream".to_string()),
        inputs: vec![PortMetadata {
            name: "Input".to_string(),
            data_type: "number".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Stats".to_string(),
            data_type: "json".to_string()
        }]
    }
);

simple_factory!(
    WindowNodeFactory,
    crate::components::manipulation::WindowConfig,
    NodeMetadata {
        id: "window".to_string(),
        name: "Window / Batch".to_string(),
        category: "Transform".to_string(),
        description: Some("Group items into windows".to_string()),
        inputs: vec![PortMetadata {
            name: "Item".to_string(),
            data_type: "any".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Window".to_string(),
            data_type: "array".to_string()
        }]
    }
);

simple_factory!(
    ExpressionNodeFactory,
    crate::components::manipulation::ExpressionConfig,
    NodeMetadata {
        id: "expression".to_string(),
        name: "Math Expression".to_string(),
        category: "Math".to_string(),
        description: Some("Evaluate math expression".to_string()),
        inputs: vec![PortMetadata {
            name: "In".to_string(),
            data_type: "number".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Result".to_string(),
            data_type: "number".to_string()
        }]
    }
);

simple_factory!(
    CheckpointNodeFactory,
    crate::components::control::CheckpointConfig,
    NodeMetadata {
        id: "checkpoint".to_string(),
        name: "Checkpoint".to_string(),
        category: "Utils".to_string(),
        description: Some("Save state".to_string()),
        inputs: vec![PortMetadata {
            name: "Exec".to_string(),
            data_type: "flow".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Exec".to_string(),
            data_type: "flow".to_string()
        }]
    }
);

simple_factory!(
    XmlNodeFactory,
    crate::components::connectors::XmlConfig,
    NodeMetadata {
        id: "xml".to_string(),
        name: "XML Parser".to_string(),
        category: "Transform".to_string(),
        description: Some("Parse/Gen XML".to_string()),
        inputs: vec![PortMetadata {
            name: "In".to_string(),
            data_type: "string".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Out".to_string(),
            data_type: "json".to_string()
        }]
    }
);

simple_factory!(
    FtpNodeFactory,
    crate::components::connectors::FtpConfig,
    NodeMetadata {
        id: "ftp".to_string(),
        name: "FTP".to_string(),
        category: "Services".to_string(),
        description: None,
        inputs: vec![PortMetadata {
            name: "Exec".to_string(),
            data_type: "flow".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Success".to_string(),
            data_type: "flow".to_string()
        }]
    }
);

simple_factory!(
    SshNodeFactory,
    crate::components::connectors::SshConfig,
    NodeMetadata {
        id: "ssh".to_string(),
        name: "SSH".to_string(),
        category: "Services".to_string(),
        description: None,
        inputs: vec![PortMetadata {
            name: "Exec".to_string(),
            data_type: "flow".to_string()
        }],
        outputs: vec![PortMetadata {
            name: "Success".to_string(),
            data_type: "flow".to_string()
        }]
    }
);

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
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "agent".to_string(),
            name: "AI Agent".to_string(),
            category: "AI".to_string(),
            description: Some("LLM Agent".to_string()),
            inputs: vec![
                PortMetadata {
                    name: "Exec".to_string(),
                    data_type: "flow".to_string(),
                },
                PortMetadata {
                    name: "Prompt".to_string(),
                    data_type: "string".to_string(),
                },
            ],
            outputs: vec![
                PortMetadata {
                    name: "Success".to_string(),
                    data_type: "flow".to_string(),
                },
                PortMetadata {
                    name: "Response".to_string(),
                    data_type: "string".to_string(),
                },
            ],
        }
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
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "aggregate".to_string(),
            name: "Aggregate".to_string(),
            category: "Transform".to_string(),
            description: Some("Batch accumulation".to_string()),
            inputs: vec![PortMetadata {
                name: "Item".to_string(),
                data_type: "any".to_string(),
            }],
            outputs: vec![PortMetadata {
                name: "Batch".to_string(),
                data_type: "array".to_string(),
            }],
        }
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
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "rss".to_string(),
            name: "RSS Feed".to_string(),
            category: "Services".to_string(),
            description: Some("Read RSS feed".to_string()),
            inputs: vec![
                PortMetadata {
                    name: "Exec".to_string(),
                    data_type: "flow".to_string(),
                },
                PortMetadata {
                    name: "URL".to_string(),
                    data_type: "string".to_string(),
                },
            ],
            outputs: vec![PortMetadata {
                name: "Items".to_string(),
                data_type: "array".to_string(),
            }],
        }
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
    fn metadata(&self) -> NodeMetadata {
        NodeMetadata {
            id: "integration".to_string(),
            name: "Integration".to_string(),
            category: "Internal".to_string(),
            description: None,
            inputs: vec![],
            outputs: vec![],
        }
    }
}

// System to register nodes
pub fn register_core_nodes(mut registry: ResMut<crate::resources::registry::NodeRegistry>) {
    println!("DEBUG: registering core nodes");
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
