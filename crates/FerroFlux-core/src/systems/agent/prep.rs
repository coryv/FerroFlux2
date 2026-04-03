use crate::components::pipeline::ExecutionContext;
use crate::components::pipeline::ReadyToExecute;
use crate::components::{
    AgentConfig, ExpectedOutput, Inbox, NodeConfig, Outbox, PinnedOutput, WorkDone,
};
use crate::integrations::registry::IntegrationRegistry;
use crate::resources::templates::CelEngine;
use crate::secrets::{DatabaseSecretStore, SecretStore};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(SystemParam)]
pub struct AgentContext<'w> {
    store: Res<'w, BlobStore>,
    registry: Res<'w, IntegrationRegistry>,
    cel_engine: Res<'w, CelEngine>,
    secret_store: Res<'w, DatabaseSecretStore>,
    work_done: ResMut<'w, WorkDone>,
    event_bus: Res<'w, crate::api::events::SystemEventBus>,
    runtime: Res<'w, crate::resources::TokioRuntime>,
    backend: Res<'w, crate::traits::execution::BackendResource>,
}

#[tracing::instrument(skip(query, ctx))]
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn agent_prep(
    mut query: Query<
        (
            Entity,
            &AgentConfig,
            &NodeConfig,
            Option<&PinnedOutput>,
            Option<&ExpectedOutput>,
            &mut Inbox,
            &mut Outbox,
        ),
        Without<ReadyToExecute>,
    >,
    mut ctx: AgentContext,
) {
    for (entity, config, node_config, pinned_opt, expected_opt, mut inbox, mut outbox) in
        query.iter_mut()
    {
        if inbox.queue.is_empty() {
            continue;
        }

        // Check for Pinned Output
        if let Some(pinned) = pinned_opt {
            while inbox.queue.pop_front().is_some() {
                tracing::info!(entity = ?entity, "Node is PINNED. Skipping execution.");
                outbox.queue.push_back((None, pinned.0.clone()));
                ctx.work_done.0 = true;
            }
            continue;
        }

        if let Some((_port, ticket)) = inbox.queue.pop_front() {
            ctx.work_done.0 = true;

            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            // Retrieve Input
            let payload_bytes = match ctx.store.claim(&ticket) {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::error!(error = %e, "Error claiming ticket");
                    continue;
                }
            };

            let input_json: Value = serde_json::from_slice(&payload_bytes).unwrap_or(json!({}));

            // Lookup Integration
            let integration_def = match ctx.registry.definitions.get(&config.provider) {
                Some(p) => p,
                None => {
                    let _ = ctx
                        .event_bus
                        .0
                        .send(crate::api::events::SystemEvent::NodeError {
                            trace_id: trace_id.clone(),
                            node_id: node_config.id,
                            error: format!("Integration '{}' not found", config.provider),
                            timestamp: chrono::Utc::now().timestamp(),
                        });
                    continue;
                }
            };

            let action_name = "chat_completion";
            let action_def = match integration_def.actions.get(action_name) {
                Some(a) => a,
                None => continue,
            };

            let tenant = node_config.tenant_id.clone();

            // Resolve Secret (Async -> Sync block)
            let rt = ctx.runtime.clone();
            let ss = ctx.secret_store.clone();
            let t_clone = tenant.clone();

            let api_key = tokio::task::block_in_place(move || {
                rt.0.block_on(async {
                    if let Some(slug) = &config.connection_slug {
                        match ss.resolve_connection(&t_clone, slug).await {
                            Ok(json_val) => json_val
                                .get("api_key")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            Err(_) => None,
                        }
                    } else {
                        let var = integration_def
                            .verify_params
                            .get("api_key")
                            .cloned()
                            .unwrap_or("API_KEY".to_string());
                        ss.get_secret(&t_clone, &var).await.ok()
                    }
                })
            })
            .unwrap_or_default();

            // Prepare Context
            let mut context_json = input_json.clone();

            // Render user prompt
            let user_prompt_template = input_json
                .get("user_prompt")
                .and_then(|v| v.as_str())
                .unwrap_or(&config.user_prompt_template);
            let user_prompt = ctx
                .cel_engine
                .render(user_prompt_template, &input_json)
                .unwrap_or_else(|_| user_prompt_template.to_string());

            // Setup context with defaults and config overrides
            if let Some(obj) = context_json.as_object_mut() {
                for input in &action_def.inputs {
                    if !obj.contains_key(&input.name)
                        && let Some(def) = &input.default
                    {
                        obj.insert(input.name.clone(), def.clone());
                    }
                }

                obj.insert("model".to_string(), json!(config.model));

                // Render system instruction
                let mut system_instruction = ctx
                    .cel_engine
                    .render(&config.system_instruction, &input_json)
                    .unwrap_or_else(|_| config.system_instruction.clone());

                // Expected Output instructions
                if let Some(expected) = expected_opt
                    && !expected.aggregated_schema.is_empty()
                {
                    let mut keys: Vec<_> = expected.aggregated_schema.iter().cloned().collect();
                    keys.sort(); // Consistent ordering for tests
                    let schema_instruction =
                        format!("\nEnsure Output matches JSON schema keys: {:?}", keys);
                    system_instruction.push_str(&schema_instruction);
                }

                obj.insert("system_instruction".to_string(), json!(system_instruction));
                obj.insert("user_prompt".to_string(), json!(user_prompt));
                obj.insert("api_key".to_string(), json!(api_key));
                obj.insert("tools".to_string(), json!(config.tools));
                obj.insert("tool_choice".to_string(), json!(config.tool_choice));
            }

            // Generic messages array
            let mut messages = Vec::new();
            let system_instruction = context_json
                .get("system_instruction")
                .and_then(|v| v.as_str())
                .unwrap_or(&config.system_instruction);
            if !system_instruction.is_empty() {
                messages.push(json!({"role": "system", "content": system_instruction}));
            }
            if let Some(hist) = context_json.get("history").and_then(|h| h.as_array()) {
                for msg in hist {
                    messages.push(msg.clone());
                }
            }
            messages.push(json!({"role": "user", "content": user_prompt}));
            if let Some(obj) = context_json.as_object_mut() {
                obj.insert("messages".to_string(), json!(messages));
            }

            // Message Transform
            let history_string = if let Some(transform_template) = &action_def.message_transform {
                ctx.cel_engine
                    .render(transform_template, &context_json)
                    .unwrap_or_else(|_| json!(messages).to_string())
            } else {
                json!(messages).to_string()
            };
            if let Some(obj) = context_json.as_object_mut() {
                obj.insert("history".to_string(), json!(history_string));
            }

            // Render Body, Path, Headers
            let body = if let Some(tpl) = &action_def.implementation.config.body_template {
                ctx.cel_engine
                    .render(tpl, &context_json)
                    .unwrap_or_else(|_| "{}".to_string())
            } else {
                "{}".to_string()
            };

            let path = ctx
                .cel_engine
                .render(&action_def.implementation.config.path, &context_json)
                .unwrap_or_else(|_| action_def.implementation.config.path.clone());
            let url = format!("{}{}", integration_def.base_url, path);

            let mut headers = std::collections::HashMap::new();
            for (k, v) in &action_def.implementation.config.headers {
                if let Ok(val) = ctx.cel_engine.render(v, &context_json) {
                    headers.insert(k.clone(), val);
                }
            }

            let method = action_def.implementation.config.method.clone();

            let job = crate::traits::execution::ExecutionJob {
                entity,
                method,
                url,
                headers,
                body,
                trace_id: trace_id.clone(),
                context: ExecutionContext {
                    provider_name: config.provider.clone(),
                    model_name: config.model.clone(),
                    node_id: node_config.id,
                    result_key: config.result_key.clone(),
                    output_transform: action_def.output_transform.as_ref().map(|t| t.text.clone()),
                    input_json: input_json.clone(),
                    start_time: chrono::Utc::now().timestamp_millis() as u64,
                },
            };

            if let Err(e) = ctx.backend.0.dispatch(job) {
                tracing::error!(error = %e, "Failed to dispatch agent execution job");
            } else {
                tracing::info!(node_id = %node_config.id, trace_id = %trace_id, model = %config.model, "Agent prep dispatched");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::events::SystemEventBus;
    use crate::components::{AgentConfig, Inbox, NodeConfig, Outbox};
    use crate::integrations::registry::{
        ActionImplementation, AuthType, IntegrationAction, IntegrationConfig, IntegrationDef,
        IntegrationRegistry,
    };
    use crate::resources::templates::CelEngine;
    use crate::secrets::DatabaseSecretStore;
    use crate::store::BlobStore;
    use crate::store::database::PersistentStore;
    use crate::traits::execution::{
        BackendResource, LocalExecutionBackend, LocalExecutionReceiver, flush_local_execution_jobs,
    };
    use ferroflux_iam::TenantId;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_agent_prep_rendering() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems((agent_prep, flush_local_execution_jobs).chain());

        // Execution Backend
        let (tx, rx) = async_channel::unbounded();
        let backend = Arc::new(LocalExecutionBackend::new(tx));
        world.insert_resource(BackendResource(backend));
        world.insert_resource(LocalExecutionReceiver(rx));

        // Resources
        let store = BlobStore::default();
        world.insert_resource(store.clone());

        let mut registry = IntegrationRegistry::default();
        let action_def = IntegrationAction {
            inputs: vec![],
            outputs: vec![],
            category: None,
            subcategory: None,
            documentation: None,
            message_transform: None,
            output_transform: None,
            implementation: ActionImplementation {
                impl_type: "http".to_string(),
                config: IntegrationConfig {
                    method: "POST".to_string(),
                    path: "'/chat'".to_string(),
                    headers: HashMap::new(),
                    body_template: Some("'{\"prompt\": \"' + user_prompt + '\"}'".to_string()),
                },
            },
        };

        let mut def = IntegrationDef {
            name: "test_provider".to_string(),
            base_url: "https://api.test.com".to_string(),
            icon_url: None,
            auth: None,
            connection_schema: None,
            verify_params: HashMap::new(),
            verify_endpoint: None,
            capabilities: None,
            actions: HashMap::new(),
            utilities: HashMap::new(),
            resources: HashMap::new(),
            auth_type: AuthType::None,
        };
        def.actions
            .insert("chat_completion".to_string(), action_def);
        registry
            .definitions
            .insert("test_provider".to_string(), def);
        world.insert_resource(registry);

        world.insert_resource(CelEngine);
        world.insert_resource(WorkDone::default());

        // Mock Secret Store
        let rt = tokio::runtime::Runtime::new().unwrap();
        world.insert_resource(crate::resources::TokioRuntime(rt.handle().clone()));
        let db = rt.block_on(async { PersistentStore::new("sqlite::memory:").await.unwrap() });
        world.insert_resource(DatabaseSecretStore::new(db, vec![0u8; 32]));

        let (event_tx, _) = tokio::sync::broadcast::channel(10);
        world.insert_resource(SystemEventBus(event_tx));

        // Entity
        let ticket = store
            .check_in(br#"{"user_prompt": "Hello World"}"#)
            .unwrap();
        let mut inbox = Inbox::default();
        inbox.queue.push_back((None, ticket));

        let entity = world
            .spawn((
                AgentConfig {
                    provider: "test_provider".to_string(),
                    model: "test-model".to_string(),
                    user_prompt_template: "user_prompt".to_string(),
                    ..Default::default()
                },
                NodeConfig {
                    id: Uuid::new_v4(),
                    name: "Test Node".to_string(),
                    node_type: "Agent".to_string(),
                    workflow_id: "global".to_string(),
                    tenant_id: TenantId::from("default_tenant"),
                },
                inbox,
                Outbox::default(),
            ))
            .id();

        // Run
        schedule.run(&mut world);

        // Verify
        let ready = world
            .get::<ReadyToExecute>(entity)
            .expect("ReadyToExecute component missing");
        assert_eq!(ready.url, "https://api.test.com/chat");
        assert!(ready.body.contains("Hello World"));
        assert_eq!(ready.context.provider_name, "test_provider");
    }
}
