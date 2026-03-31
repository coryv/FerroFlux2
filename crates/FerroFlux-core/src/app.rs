use crate::api::{ApiCommand, ApiReceiver};
use crate::components::{AgentConcurrency, WorkDone};
use crate::nodes::register_core_nodes;
use crate::resources::GlobalHttpClient;
use crate::store::BlobStore;
use crate::store::analytics::AnalyticsBackend;
use crate::store::database::PersistentStore;
use crate::systems::api_worker::api_command_worker;
use crate::systems::janitor::JanitorTimer;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;

use std::path::PathBuf;
use std::sync::Arc;

/// Output of [`AppBuilder::build`] — all resources the caller needs to drive the runtime.
///
/// Named fields replace the opaque 8-element tuple that was previously returned, making
/// call sites self-documenting and immune to positional argument confusion.
pub struct AppContext {
    /// The ECS runtime (world + schedule). Call `.run().await` to start, or `.update()` to tick.
    pub app: App,
    /// Channel for sending API commands to the ECS loop.
    pub api_tx: async_channel::Sender<ApiCommand>,
    /// Broadcast sender for system events. Clone/subscribe to receive telemetry or SSE feeds.
    pub event_tx: tokio::sync::broadcast::Sender<crate::api::events::SystemEvent>,
    /// Handle to the SQLite persistent store (passed to the API server for workflow CRUD).
    pub store: PersistentStore,
    /// In-memory blob store (returned so the API server can share the same instance).
    pub blob_store: BlobStore,
    /// The AES-GCM master encryption key (needed by the API to decrypt connections).
    pub master_key: Vec<u8>,
    /// Loaded integration definitions.
    pub integration_registry: crate::integrations::IntegrationRegistry,
    /// Per-request integration response cache.
    pub action_cache: crate::store::cache::IntegrationCache,
    /// Receiver for trigger events.
    pub trigger_rx: async_channel::Receiver<ferroflux_types::trigger::TriggerEvent>,
}

pub struct AppBuilder {
    db_url: Option<String>,
    store: Option<PersistentStore>,
    master_key: Option<Vec<u8>>,
    import_flows: bool,
    analytics_backend: Option<Arc<dyn AnalyticsBackend>>,
    execution_backend: Option<Arc<dyn crate::traits::execution::ExecutionBackend>>,
    trigger_providers: Vec<Box<dyn crate::traits::trigger::TriggerProvider>>,
    plugins: Vec<Box<dyn ferroflux_types::Plugin>>,
    platforms_path: Option<PathBuf>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CorePlugin;

impl ferroflux_types::Plugin for CorePlugin {
    fn build(&self, _world: &mut World, schedule: &mut Schedule) {
        crate::systems::register_core_systems(schedule);
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        let builder = Self {
            db_url: None,
            store: None,
            master_key: None,
            import_flows: true,
            analytics_backend: None,
            execution_backend: None,
            trigger_providers: Vec::new(),
            plugins: Vec::new(),
            platforms_path: None,
        };
        builder.add_plugin(CorePlugin)
    }

    pub fn add_plugin<P: ferroflux_types::Plugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn with_platforms_path(mut self, path: PathBuf) -> Self {
        self.platforms_path = Some(path);
        self
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

    pub fn with_execution_backend(
        mut self,
        backend: Arc<dyn crate::traits::execution::ExecutionBackend>,
    ) -> Self {
        self.execution_backend = Some(backend);
        self
    }

    pub fn with_trigger_provider(
        mut self,
        provider: Box<dyn crate::traits::trigger::TriggerProvider>,
    ) -> Self {
        self.trigger_providers.push(provider);
        self
    }

    /// Builds the App synchronously, returning an [`App`] that can be manually ticked.
    /// This skips spawning the background command worker thread.
    pub fn build_sync(self) -> anyhow::Result<App> {
        // 1. Channel for API -> ECS (dummy or manual)
        let (_api_tx, api_rx) = async_channel::unbounded::<ApiCommand>();

        // 2. Event Bus
        let (event_tx, _) = tokio::sync::broadcast::channel::<crate::api::events::SystemEvent>(100);

        // 3. Store
        let store = if let Some(s) = self.store {
            s
        } else {
            let url = self.db_url.unwrap_or("sqlite::memory:".to_string());
            // SAFETY: In sync mode, we assume the caller is in an async runtime (like tokio::test)
            // or we use block_on. AppBuilder::build is already async.
            // For build_sync, we'll try to get handle or block.
            match tokio::runtime::Handle::try_current() {
                Ok(h) => {
                    let url_clone = url.clone();
                    tokio::task::block_in_place(move || h.block_on(PersistentStore::new(&url_clone)))?
                }
                Err(_) => tokio::runtime::Runtime::new()?.block_on(PersistentStore::new(&url))?,
            }
        };

        // 4. BlobStore
        let blob_store = BlobStore::default();

        // 5. Integration Registry
        use crate::integrations::IntegrationRegistry;
        let mut int_registry = IntegrationRegistry::default();
        let _ = int_registry.load_from_directory("integrations");

        // 6. Master Key
        let master_key = self.master_key.unwrap_or_else(|| {
            ferroflux_security::encryption::get_or_create_master_key()
                .expect("Failed to get master key")
        });

        // 8. ECS World Setup
        let mut world = World::new();
        let mut schedule = Schedule::default();

        world.insert_resource(blob_store.clone());
        world.insert_resource(ApiReceiver(api_rx));
        world.insert_resource(GlobalHttpClient::default());
        world.insert_resource(crate::resources::AgentResultChannel::default());
        world.insert_resource(crate::resources::HttpResultChannel::default());
        world.insert_resource(ferroflux_types::resources::SseTriggerRegistry::default());
        world.insert_resource(crate::api::events::SystemEventBus(event_tx.clone()));

        // Trigger Protocol Setup
        let (trigger_tx, trigger_rx) = async_channel::unbounded::<ferroflux_types::trigger::TriggerEvent>();
        let trigger_sender_res = ferroflux_types::trigger::TriggerSender(trigger_tx.clone());
        
        world.insert_resource(trigger_sender_res);
        // We don't necessarily need receiver in sync app, but good for completeness
        world.insert_resource(ferroflux_types::trigger::TriggerReceiver(trigger_rx));

        world.insert_resource(store.clone());

        // Use current runtime handle
        if let Ok(h) = tokio::runtime::Handle::try_current() {
            world.insert_resource(crate::resources::TokioRuntime(h));
        }

        // Execution Backend Setup
        if let Some(backend) = self.execution_backend {
            world.insert_resource(crate::traits::execution::BackendResource(backend));
        } else {
            // Default Local Backend
            let (tx, rx) = async_channel::unbounded();
            let backend =
                std::sync::Arc::new(crate::traits::execution::LocalExecutionBackend::new(tx));
            world.insert_resource(crate::traits::execution::BackendResource(backend));
            world.insert_resource(crate::traits::execution::LocalExecutionReceiver(rx));
        }

        // Registry
        world.insert_resource(int_registry.clone());
        world.insert_resource(JanitorTimer::default());
        world.insert_resource(WorkDone::default());
        world.insert_resource(crate::resources::NodeRouter::default());
        world.insert_resource(AgentConcurrency(Arc::new(tokio::sync::Semaphore::new(50))));
        world.insert_resource(crate::resources::GraphTopology::default());
        world.insert_resource(crate::resources::templates::CelEngine);
        world.insert_resource(crate::resources::PipelineResultChannel::default());
        
        let tool_registry = crate::tools::ToolRegistry::default();
        world.insert_resource(tool_registry);

        world.insert_resource(crate::resources::NodeRegistry::default());

        // 9. YAML Definitions Registry
        let mut def_registry = crate::resources::DefinitionRegistry::default();

        let platform_path = self.platforms_path.unwrap_or_else(|| {
            let mut p = std::path::PathBuf::from("platforms");
            if !p.exists() {
                let mut curr = std::env::current_dir().unwrap_or_default();
                for _ in 0..5 {
                    let candidate = curr.join("platforms");
                    if candidate.exists() {
                        p = candidate;
                        break;
                    }
                    if let Some(parent) = curr.parent() {
                        curr = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
            p
        });

        if platform_path.exists() {
            let _ = def_registry.load_from_dir(&platform_path);
        }
        world.insert_resource(crate::api::PlatformPath(platform_path.clone()));
        world.insert_resource(def_registry.clone());

        // 10. Bridge YAML to NodeRegistry
        {
            let mut system_state =
                SystemState::<ResMut<crate::resources::NodeRegistry>>::new(&mut world);
            let mut registry_res = system_state.get_mut(&mut world);

            register_core_nodes(&mut registry_res);

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
            master_key.clone(),
        ));

        // OAuth2 Token Refresh Locks
        world.insert_resource(ferroflux_db::oauth2::TokenRefreshLocks::default());

        // 11. Run Plugins
        for plugin in self.plugins {
            plugin.build(&mut world, &mut schedule);
        }

        // Add flush_local_execution_jobs if local receiver exists
        if world.contains_resource::<crate::traits::execution::LocalExecutionReceiver>() {
            schedule.add_systems(crate::traits::execution::flush_local_execution_jobs);
        }
        schedule.add_systems(api_command_worker);

        Ok(App { world, schedule })
    }

    /// Builds the App and returns a structured [`AppContext`] with all runtime handles.
    pub async fn build(self) -> anyhow::Result<AppContext> {
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
        world.insert_resource(ferroflux_types::resources::SseTriggerRegistry::default());
        world.insert_resource(crate::api::events::SystemEventBus(event_tx.clone()));

        // Trigger Protocol Setup
        let (trigger_tx, trigger_rx) = async_channel::unbounded::<ferroflux_types::trigger::TriggerEvent>();
        let trigger_sender_res = ferroflux_types::trigger::TriggerSender(trigger_tx.clone());
        let trigger_receiver_res = ferroflux_types::trigger::TriggerReceiver(trigger_rx.clone());
        
        world.insert_resource(trigger_sender_res);
        world.insert_resource(trigger_receiver_res);

        // Initialize Trigger Providers
        for provider in self.trigger_providers {
            tracing::info!("Initializing trigger provider: {}", provider.name());
            provider.on_enable(ferroflux_types::trigger::TriggerSender(trigger_tx.clone()));
        }

        world.insert_resource(store.clone());

        // Use current runtime handle
        let runtime_handle = tokio::runtime::Handle::current();
        world.insert_resource(crate::resources::TokioRuntime(runtime_handle));

        // Execution Backend Setup
        if let Some(backend) = self.execution_backend {
            world.insert_resource(crate::traits::execution::BackendResource(backend));
        } else {
            // Default Local Backend
            let (tx, rx) = async_channel::unbounded();
            let backend =
                std::sync::Arc::new(crate::traits::execution::LocalExecutionBackend::new(tx));
            world.insert_resource(crate::traits::execution::BackendResource(backend));
            world.insert_resource(crate::traits::execution::LocalExecutionReceiver(rx));
        }

        // Registry
        world.insert_resource(int_registry.clone());
        world.insert_resource(JanitorTimer::default());
        world.insert_resource(WorkDone::default());
        world.insert_resource(crate::resources::NodeRouter::default());
        world.insert_resource(AgentConcurrency(Arc::new(tokio::sync::Semaphore::new(50))));
        world.insert_resource(crate::resources::GraphTopology::default());
        world.insert_resource(crate::resources::templates::CelEngine);
        world.insert_resource(crate::resources::PipelineResultChannel::default());
        // Create and register ToolRegistry
        let tool_registry = crate::tools::ToolRegistry::default();
        world.insert_resource(tool_registry);

        world.insert_resource(crate::resources::NodeRegistry::default());

        // 9. YAML Definitions Registry
        let mut def_registry = crate::resources::DefinitionRegistry::default();

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
            match def_registry.load_from_dir(&platform_path) {
                Err(e) => {
                    tracing::error!(path = ?platform_path, error = %e, "Failed to load YAML platforms");
                }
                Ok(validation) if validation.has_errors() => {
                    for diag in &validation.diagnostics {
                        if diag.severity == ferroflux_integration::Severity::Error {
                            tracing::warn!(rule = diag.rule, message = %diag.message, "Integration validation error");
                        }
                    }
                }
                Ok(_) => {}
            }
        } else {
            tracing::warn!("'platforms' directory not found, skipping YAML definitions");
        }
        world.insert_resource(crate::api::PlatformPath(platform_path.clone()));
        world.insert_resource(def_registry.clone());

        // 10. Bridge YAML to NodeRegistry
        {
            let mut system_state =
                SystemState::<ResMut<crate::resources::NodeRegistry>>::new(&mut world);
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

        // OAuth2 Token Refresh Locks
        world.insert_resource(ferroflux_db::oauth2::TokenRefreshLocks::default());

        // 11. Run Plugins
        for plugin in self.plugins {
            plugin.build(&mut world, &mut schedule);
        }

        // Add flush_local_execution_jobs if local receiver exists
        if world.contains_resource::<crate::traits::execution::LocalExecutionReceiver>() {
            schedule.add_systems(crate::traits::execution::flush_local_execution_jobs);
        }
        schedule.add_systems(api_command_worker);

        Ok(AppContext {
            app: App { world, schedule },
            api_tx,
            event_tx,
            store: store_server,
            blob_store: blob_store_server,
            master_key: master_key_clone,
            integration_registry: int_registry,
            action_cache,
            trigger_rx,
        })
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

    pub fn shutdown(&mut self) {
        if let Some(mut registry) = self
            .world
            .get_resource_mut::<ferroflux_types::resources::SseTriggerRegistry>()
        {
            registry.abort_all();
        }
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
