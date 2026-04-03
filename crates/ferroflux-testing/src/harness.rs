use ferroflux_core::app::{App, AppBuilder};
use ferroflux_core::graph_loader::load_graph_from_str;
use ferroflux_iam::TenantId;
use ferroflux_types::registry::NodeRegistry;
use ferroflux_core::resources::{
    DefinitionRegistry, WorkDone,
};
use ferroflux_core::tools::ToolRegistry;
use serde_json::json;
use std::path::PathBuf;
use wiremock::MockServer;

/// Test Harness for Integration Testing
pub struct TestHarness {
    pub app: App,
    pub server: MockServer,
    pub tenant: TenantId,
}

impl TestHarness {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        
        let mut tool_registry = ToolRegistry::default();
        ferroflux_tools::register_core_tools(&mut tool_registry);

        // build_sync returns a single-threaded Bevy App that we can manually tick
        let mut app = AppBuilder::new()
            .build_sync()
            .expect("Failed to build sync App for TestHarness");

        // Override with our tool-populated registry
        app.world.insert_resource(tool_registry);

        Self {
            app,
            server,
            tenant: TenantId::from("default_tenant"),
        }
    }

    pub fn tick(&mut self) {
        self.app.update();
    }

    pub fn load_platforms(&mut self) -> anyhow::Result<()> {
        let mut platform_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        platform_path.pop();
        platform_path.pop();
        platform_path.push("platforms");

        if !platform_path.exists() {
            tracing::warn!("Platforms directory not found at {:?}", platform_path);
            return Ok(());
        }

        // Get DefinitionRegistry from world
        let defs = {
            let mut def_registry = self.app.world.resource_mut::<DefinitionRegistry>();
            if let Err(e) = def_registry.0.load_from_dir(&platform_path) {
                tracing::error!(path = ?platform_path, error = %e, "Initial platforms load failed");
            }
            def_registry.0.definitions.clone()
        };

        // Bridge to NodeRegistry
        {
            let mut registry = self.app.world.resource_mut::<NodeRegistry>();
            for (id, def) in &defs {
                registry.register(
                    id,
                    Box::new(ferroflux_core::nodes::yaml_factory::YamlNodeFactory::new(
                        def.clone(),
                    )),
                );
            }
        }

        Ok(())
    }

    pub fn load_waml(&mut self, waml: &str) -> anyhow::Result<()> {
        load_graph_from_str(&mut self.app.world, self.tenant.clone(), waml)
    }

    pub async fn run_until_idle(&mut self, max_ticks: usize) {
        for _ in 0..max_ticks {
            self.tick();
            let work_done = self.app.world.resource::<WorkDone>().0;
            if !work_done {
                break;
            }
            // NO reset of work_done.0 here - the engine/systems should set it
            tokio::task::yield_now().await;
        }
    }

    /// Overrides a configuration value for a registered platform.
    pub fn set_platform_config(&mut self, platform_id: &str, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        let mut def_registry = self.app.world.resource_mut::<DefinitionRegistry>();
        
        if let Some(platform) = def_registry.0.platforms.get_mut(platform_id) {
            platform.config.insert(key.to_string(), value);
            Ok(())
        } else {
            anyhow::bail!("Platform not found: {}", platform_id)
        }
    }

    /// Overrides a global workflow configuration value.
    pub fn set_workflow_config(&mut self, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        use ferroflux_core::components::execution_state::WorkflowDefinition;
        let mut query = self.app.world.query::<&mut WorkflowDefinition>();
        if let Some(mut def) = query.iter_mut(&mut self.app.world).next() {
            def.config.insert(key.to_string(), value);
            Ok(())
        } else {
            anyhow::bail!("WorkflowDefinition not found in world")
        }
    }

    /// Add an integration and automatically point its base_url to the mock server.
    pub fn add_mocked_integration(&mut self, yaml: &str) -> anyhow::Result<String> {
        let def: ferroflux_integration::definition::PlatformDefinition = serde_yaml::from_str(yaml)?;
        let id = def.meta.id.clone();
        
        let mut def_registry = self.app.world.resource_mut::<DefinitionRegistry>();
        def_registry.0.platforms.insert(id.clone(), def);
        
        // Point base_url to mock server
        if let Some(p) = def_registry.0.platforms.get_mut(&id) {
            p.config.insert("base_url".to_string(), json!(self.server.uri()));
        }
        
        Ok(id)
    }

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
