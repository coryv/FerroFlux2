use crate::api::{ApiCommand, ApiReceiver};
use crate::components::{AgentConcurrency, WorkDone};
use crate::nodes::register_core_nodes;
use crate::resources::GlobalHttpClient;
use crate::store::BlobStore;
use crate::store::analytics::{AnalyticsBackend, NoopStore};
use crate::store::batcher::AnalyticsBatcher;
use crate::store::database::PersistentStore;
use crate::systems::api_worker::api_command_worker;
use crate::systems::compute::WasmRuntime;
use crate::systems::gateway;
use crate::systems::janitor::JanitorTimer;
use crate::systems::register_core_systems;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use rhai::Engine;
use std::sync::Arc;

pub struct AppBuilder {
    db_url: Option<String>,
    store: Option<PersistentStore>,
    master_key: Option<Vec<u8>>,
    import_flows: bool,
    analytics_backend: Option<Arc<dyn AnalyticsBackend>>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            db_url: None,
            store: None,
            master_key: None,
            import_flows: true,
            analytics_backend: None,
        }
    }

    pub fn with_db_url(mut self, url: impl Into<String>) -> Self {
        self.db_url = Some(url.into());
        self
    }

    pub async fn with_production_store(mut self) -> anyhow::Result<Self> {
        let db_url = std::env::var("DATABASE_URL").unwrap_or("sqlite:ferroflux.db".to_string());

        // Ensure sqlite file exists
        if !std::path::Path::new("ferroflux.db").exists() {
            std::fs::File::create("ferroflux.db").ok();
        }

        let store = PersistentStore::new(&db_url).await?;
        self.store = Some(store);
        self.db_url = Some(db_url);
        Ok(self)
    }

    pub fn with_store(mut self, store: PersistentStore) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_master_key(mut self, key: Vec<u8>) -> Self {
        self.master_key = Some(key);
        self
    }

    pub fn with_analytics_backend(mut self, backend: Arc<dyn AnalyticsBackend>) -> Self {
        self.analytics_backend = Some(backend);
        self
    }

    /// Builds the App and returns the App instance along with channels for external communication.
    pub async fn build(
        self,
    ) -> anyhow::Result<(
        App,
        async_channel::Sender<ApiCommand>,
        tokio::sync::broadcast::Sender<crate::api::events::SystemEvent>,
        PersistentStore,
        BlobStore,
        Vec<u8>,
        crate::integrations::IntegrationRegistry,
        crate::store::cache::IntegrationCache,
        Arc<AnalyticsBatcher>,
    )> {
        // 1. Channel for API -> ECS
        let (api_tx, api_rx) = async_channel::unbounded::<ApiCommand>();

        // 2. Event Bus
        let (event_tx, _) = tokio::sync::broadcast::channel::<crate::api::events::SystemEvent>(100);

        // 3. Store
        let store = if let Some(s) = self.store {
            s
        } else {
            let url = self.db_url.unwrap_or("sqlite::memory:".to_string());
            PersistentStore::new(&url).await?
        };

        // 3.5 Auto-seeding flows
        if self.import_flows {
            let default_tenant = ferroflux_iam::TenantId::from("default_tenant");
            let workflows = store
                .load_active_workflows(&default_tenant)
                .await
                .unwrap_or_default();

            for (id, json, _status) in workflows {
                match api_tx
                    .send(ApiCommand::LoadGraph(default_tenant.clone(), json))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!(workflow_id = %id, error = %e, "Failed to queue restored workflow")
                    }
                }
            }
        }

        let store_server = store.clone();

        // 4. BlobStore
        let blob_store = BlobStore::default();
        let blob_store_server = blob_store.clone();

        // 5. Integration Registry
        use crate::integrations::IntegrationRegistry;
        let mut int_registry = IntegrationRegistry::default();
        let _ = int_registry.load_from_directory("integrations"); // Ignore errors or count

        // 6. Master Key
        let master_key = self.master_key.unwrap_or_else(|| {
            ferroflux_security::encryption::get_or_create_master_key()
                .expect("Failed to get master key")
        });
        let master_key_clone = master_key.clone();

        // 6.5 Analytics Setup
        let backend = self
            .analytics_backend
            .unwrap_or_else(|| Arc::new(NoopStore));
        let analytics = Arc::new(AnalyticsBatcher::new(backend));

        // 7. API Server components (returned, not spawned)
        let action_cache = crate::store::cache::IntegrationCache::default();

        // 8. ECS World Setup
        let mut world = World::new();
        let mut schedule = Schedule::default();

        world.insert_resource(blob_store.clone());
        world.insert_resource(ApiReceiver(api_rx));
        world.insert_resource(GlobalHttpClient::default());
        world.insert_resource(crate::resources::AgentResultChannel::default());
        world.insert_resource(crate::resources::HttpResultChannel::default());
        world.insert_resource(crate::api::events::SystemEventBus(event_tx.clone()));
        world.insert_resource(store.clone());

        // Heavy resources
        let engine = Engine::new();
        world.insert_non_send_resource(engine);

        // Use current runtime handle
        let runtime_handle = tokio::runtime::Handle::current();
        world.insert_resource(crate::resources::TokioRuntime(runtime_handle));

        // Registry
        world.insert_resource(int_registry.clone());
        world.insert_resource(WasmRuntime::default());
        world.insert_resource(JanitorTimer::default());
        world.insert_resource(WorkDone::default());
        world.insert_resource(crate::resources::NodeRouter::default());
        world.insert_resource(AgentConcurrency(Arc::new(tokio::sync::Semaphore::new(50))));
        world.insert_resource(crate::resources::GraphTopology::default());
        world.insert_resource(crate::resources::templates::TemplateEngine::default());
        world.insert_resource(crate::resources::PipelineResultChannel::default());
        // Create and register ToolRegistry
        let mut tool_registry = crate::tools::registry::ToolRegistry::default();
        crate::tools::register_core_tools(&mut tool_registry);
        world.insert_resource(tool_registry);

        world.insert_resource(crate::resources::registry::NodeRegistry::default());

        // 9. YAML Definitions Registry
        let mut def_registry = crate::resources::registry::DefinitionRegistry::default();

        // Robust discovery of the 'platforms' directory
        let mut platform_path = std::path::PathBuf::from("platforms");
        if !platform_path.exists() {
            // Try searching up for workspace root platforms
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

        if platform_path.exists() {
            if let Err(e) = def_registry.load_from_dir(&platform_path) {
                tracing::error!(path = ?platform_path, error = %e, "Failed to load YAML platforms");
            }
        } else {
            tracing::warn!("'platforms' directory not found, skipping YAML definitions");
        }
        world.insert_resource(crate::api::PlatformPath(platform_path.clone()));
        world.insert_resource(def_registry.clone());

        // 10. Bridge YAML to NodeRegistry
        {
            let mut system_state =
                SystemState::<ResMut<crate::resources::registry::NodeRegistry>>::new(&mut world);
            let mut registry_res = system_state.get_mut(&mut world);

            // Register Core Nodes (Hardcoded)
            register_core_nodes(&mut registry_res);

            // Register YAML-defined Nodes
            for (id, def) in &def_registry.definitions {
                registry_res.register(
                    id,
                    Box::new(crate::nodes::yaml_factory::YamlNodeFactory::new(
                        def.clone(),
                    )),
                );
            }
        }

        // Secrets
        world.insert_resource(crate::secrets::DatabaseSecretStore::new(
            store.clone(),
            master_key_clone.clone(),
        ));

        // Webhook Queue Initialization (Manual for now, since server is external)
        // But the ingest_worker is registered below.
        // We need to ensure the channel is set up.
        // In gateway.rs we have a lazy static. We should initialize it here?
        // But gateway.rs in core is static.
        // We can expose a function in gateway to init the queue?
        // Or we can just let the external server call init?
        // Actually, gateway::run_webhook_server used to init it.
        // We need a way to initialize the queue channel.
        let (wh_tx, wh_rx) = async_channel::unbounded();
        gateway::WEBHOOK_QUEUE.set((wh_tx.clone(), wh_rx)).ok();
        // Return the webhook tx to Caller?
        // Or caller can send to gateway system?
        // The external server needs `wh_tx` (or ability to get it).
        // Since it's a static in generic lib, if the external app links to this lib, it can access the static?
        // Yes.

        // Register Core Systems
        register_core_systems(&mut schedule);
        schedule.add_systems(api_command_worker);

        Ok((
            App { world, schedule },
            api_tx,
            event_tx,
            store_server,
            blob_store_server,
            master_key_clone,
            int_registry,
            action_cache,
            analytics,
        ))
    }
}

pub struct App {
    pub world: World,
    pub schedule: Schedule,
}

impl App {
    pub fn update(&mut self) {
        self.world.resource_mut::<WorkDone>().0 = false;
        self.schedule.run(&mut self.world);
    }

    pub async fn run(mut self) {
        tracing::info!("Starting main loop");
        loop {
            self.update();

            // Always yield to async runtime to allow IO to progress
            tokio::task::yield_now().await;

            if !self.world.resource::<WorkDone>().0 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
}
