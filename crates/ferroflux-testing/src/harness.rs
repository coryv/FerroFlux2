use bevy_ecs::prelude::*;
use ferroflux_core::app::App;
use ferroflux_core::graph_loader::load_graph_from_str;
use ferroflux_iam::TenantId;
use ferroflux_types::registry::NodeRegistry;
use ferroflux_core::resources::{
    AgentConcurrency, AgentResultChannel, DefinitionRegistry, GlobalHttpClient, GraphTopology,
    HttpResultChannel, NodeRouter, PipelineResultChannel, TokioRuntime, WorkDone,
};
use ferroflux_core::nodes::register_core_nodes;
use ferroflux_core::store::BlobStore;
use ferroflux_core::api::ApiReceiver;
use ferroflux_core::systems::janitor::JanitorTimer;
use ferroflux_db::workflow::PersistentStore;
use ferroflux_core::integrations::{IntegrationDef, IntegrationRegistry};
use ferroflux_integration::definition::PlatformDefinition;
use bevy_ecs::system::SystemState;
use std::sync::Arc;
use tokio::runtime::Handle;
use wiremock::MockServer;

pub struct TestHarness {
    pub app: App,
    pub tenant: TenantId,
    pub server: MockServer,
}

impl TestHarness {
    pub async fn new() -> anyhow::Result<Self> {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        let server = MockServer::start().await;

        // 1. Channels
        let (api_tx, api_rx) = async_channel::unbounded();
        let (trigger_tx, trigger_rx) = async_channel::unbounded();
        let (event_tx, _) = tokio::sync::broadcast::channel(100);

        // 2. Core Resources
        world.insert_resource(WorkDone::default());
        world.insert_resource(NodeRouter::default());
        world.insert_resource(GraphTopology::default());
        world.insert_resource(GlobalHttpClient::default());
        world.insert_resource(TokioRuntime(Handle::current()));
        world.insert_resource(AgentResultChannel::default());
        world.insert_resource(HttpResultChannel::default());
        world.insert_resource(PipelineResultChannel::default());
        world.insert_resource(BlobStore::default());
        world.insert_resource(ApiReceiver(api_rx));
        world.insert_resource(JanitorTimer::default());
        world.insert_resource(AgentConcurrency(Arc::new(tokio::sync::Semaphore::new(50))));
        world.insert_resource(ferroflux_core::api::events::SystemEventBus(event_tx));
        world.insert_resource(ferroflux_types::trigger::TriggerSender(trigger_tx));
        world.insert_resource(ferroflux_types::trigger::TriggerReceiver(trigger_rx));
        world.insert_resource(ferroflux_types::resources::SseTriggerRegistry::default());
        world.insert_resource(ferroflux_core::resources::templates::TemplateEngine::default());
        world.insert_resource(ferroflux_db::oauth2::TokenRefreshLocks::default());

        // 3. Registries
        let mut node_registry = NodeRegistry::default();
        register_core_nodes(&mut node_registry);
        world.insert_resource(node_registry);

        let int_registry = IntegrationRegistry::default();
        world.insert_resource(int_registry);

        let def_registry = DefinitionRegistry::default();
        world.insert_resource(def_registry);
        
        let mut tool_registry = ferroflux_core::tools::ToolRegistry::default();
        ferroflux_tools::register_core_tools(&mut tool_registry);
        world.insert_resource(tool_registry);

        // 3. Persistent Store (In-Memory)
        let store = PersistentStore::new("sqlite::memory:").await?;
        world.insert_resource(store.clone());

        // Secrets
        let master_key = vec![0u8; 32];
        world.insert_resource(ferroflux_core::secrets::DatabaseSecretStore::new(
            store.clone(),
            master_key,
        ));
        
        world.insert_resource(ferroflux_db::oauth2::TokenRefreshLocks::default());

        // 4. Execution Backend (Local)
        let (tx, rx) = async_channel::unbounded();
        let backend = Arc::new(ferroflux_core::traits::execution::LocalExecutionBackend::new(tx));
        world.insert_resource(ferroflux_core::traits::execution::BackendResource(backend));
        world.insert_resource(ferroflux_core::traits::execution::LocalExecutionReceiver(rx));

        // 5. Systems
        ferroflux_core::systems::register_core_systems(&mut schedule);
        schedule.add_systems(ferroflux_core::traits::execution::flush_local_execution_jobs);
        schedule.add_systems(ferroflux_core::systems::api_worker::api_command_worker);

        let mut harness = Self {
            app: App { world, schedule },
            tenant: TenantId::from("test_tenant"),
            server,
        };

        // Auto-load platforms
        harness.load_platforms()?;

        Ok(harness)
    }

    pub fn load_platforms(&mut self) -> anyhow::Result<()> {
        let mut def_registry = DefinitionRegistry::default();
        
        let mut platform_path = std::path::PathBuf::from("platforms");
        if !platform_path.exists() {
            let mut curr = std::env::current_dir().unwrap_or_default();
            for _ in 0..5 {
                let candidate = curr.join("platforms");
                if candidate.exists() {
                    platform_path = candidate;
                    break;
                }
                if let Some(parent) = curr.parent() {
                    curr = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }

        if !platform_path.exists() {
            tracing::warn!("Platforms directory not found");
            return Ok(());
        }

        // Now that load_from_dir is more robust, I can call it once
        if let Err(e) = def_registry.0.load_from_dir(&platform_path) {
            tracing::error!(path = ?platform_path, error = %e, "Initial platforms load failed");
        }

        // Bridge to NodeRegistry
        let mut system_state = SystemState::<ResMut<NodeRegistry>>::new(&mut self.app.world);
        let mut registry = system_state.get_mut(&mut self.app.world);
        
        for (id, def) in &def_registry.0.definitions {
            registry.register(
                id,
                Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(
                    def.clone(),
                )),
            );
        }
        
        self.app.world.insert_resource(def_registry);
        Ok(())
    }

    pub fn load_waml(&mut self, waml: &str) -> anyhow::Result<()> {
        load_graph_from_str(&mut self.app.world, self.tenant.clone(), waml)
    }

    pub fn tick(&mut self) {
        self.app.update();
    }

    pub async fn run_until_idle(&mut self, max_ticks: usize) {
        for _ in 0..max_ticks {
            self.tick();
            let work_done = self.app.world.resource::<WorkDone>().0;
            if !work_done {
                tokio::task::yield_now().await;
            }
        }
    }

    /// Overrides a configuration value for a registered platform.
    pub fn set_platform_config(&mut self, platform_id: &str, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        let mut system_state = SystemState::<ResMut<DefinitionRegistry>>::new(&mut self.app.world);
        let mut def_registry = system_state.get_mut(&mut self.app.world);
        
        if let Some(platform) = def_registry.0.platforms.get_mut(platform_id) {
            platform.config.insert(key.to_string(), value);
            Ok(())
        } else {
            anyhow::bail!("Platform not found: {}", platform_id)
        }
    }

    /// Load an integration definition directly from a YAML string.
    pub fn add_integration(&mut self, yaml: &str) -> anyhow::Result<()> {
        let def: IntegrationDef = serde_yaml::from_str(yaml)?;
        let mut registry = self.app.world.resource_mut::<IntegrationRegistry>();
        registry.definitions.insert(def.name.clone(), def);
        Ok(())
    }

    /// Add an integration and automatically point its base_url to the mock server.
    pub fn add_mocked_integration(&mut self, yaml: &str) -> anyhow::Result<String> {
        let mut def: IntegrationDef = serde_yaml::from_str(yaml)?;
        let name = def.name.clone();
        def.base_url = self.server.uri();
        
        let mut registry = self.app.world.resource_mut::<IntegrationRegistry>();
        registry.definitions.insert(name.clone(), def);
        Ok(name)
    }

    /// Trigger a specific node in the current graph by its ID
    pub fn trigger_node(&mut self, node_id: uuid::Uuid, payload: serde_json::Value) -> anyhow::Result<()> {
        ferroflux_core::api::handlers::trigger::handle_trigger_node(
            &mut self.app.world,
            self.tenant.clone(),
            node_id,
            payload
        )
    }

    pub fn mock_server(&self) -> &MockServer {
        &self.server
    }
}
