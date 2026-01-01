use anyhow::Result;
use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEvent;
use ferroflux_core::app::App;
use ferroflux_core::app::AppBuilder;
use ferroflux_core::components::core::{Edge, NodeConfig};
use flow_canvas::model::{GraphState, NodeData};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

/// The SDK Client for interacting with the FerroFlux Engine.
///
/// This client manages the lifecycle of the engine, graph deployment,
/// and event synchronization, serving as the primary interface for
/// Desktop, Web, and CLI applications.
pub struct FerroFluxClient<T: NodeData> {
    /// Handle to the underlying FerroFlux engine.
    pub engine: Arc<Mutex<App>>,
    /// Channel for sending commands to the engine.
    api_tx: async_channel::Sender<ferroflux_core::api::ApiCommand>,
    /// Subscriber to the engine's event bus.
    event_rx: broadcast::Receiver<SystemEvent>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: NodeData> FerroFluxClient<T> {
    /// Initializes a new SDK client with default engine settings.
    ///
    /// This is the standard entry point for most applications.
    pub async fn init() -> Result<Self> {
        let (mut engine, api_tx, event_tx, ..) = AppBuilder::new().build().await?;

        // Register Core Tools
        if let Some(mut registry) = engine
            .world
            .get_resource_mut::<ferroflux_core::tools::registry::ToolRegistry>()
        {
            ferroflux_core::tools::register_core_tools(&mut registry);
        }

        Ok(Self::new(engine, api_tx, event_tx))
    }

    /// Creates a new SDK client for a given engine instance.
    pub fn new(
        engine: App,
        api_tx: async_channel::Sender<ferroflux_core::api::ApiCommand>,
        event_bus: broadcast::Sender<SystemEvent>,
    ) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            api_tx,
            event_rx: event_bus.subscribe(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Compiles and deploys the current Canvas state to the Engine.
    ///
    /// This process "lowers" the high-level visual graph into a set of optimized
    /// ECS entities and components ready for execution. It strips away layout
    /// information (position, size) as the engine operates purely on logic.
    pub async fn compile_and_deploy(&mut self, graph: &GraphState<T>) -> Result<()> {
        let mut engine = self.engine.lock().await;
        let world = &mut engine.world;

        // 1. Clear existing nodes/edges from the engine (simplified for V1)
        // In a real app, we might want incremental updates.
        let mut query = world.query_filtered::<Entity, With<NodeConfig>>();
        let entities: Vec<Entity> = query.iter(world).collect();
        for entity in entities {
            world.despawn(entity);
        }

        let mut canvas_to_entity = HashMap::new();

        // 2. Spawn Nodes
        for (id, node) in &graph.nodes {
            let entity = world
                .spawn(NodeConfig {
                    id: node.uuid,
                    name: format!("{:?}", node.id), // Placeholder name
                    node_type: node.data.node_type(), // This now comes from node.data
                    workflow_id: None,
                    tenant_id: None,
                })
                .id();

            canvas_to_entity.insert(id, entity);
        }

        // 3. Spawn Edges
        for (_, conn) in &graph.connections {
            let from_node_id = graph.ports.get(conn.from).map(|p| p.node);
            let to_node_id = graph.ports.get(conn.to).map(|p| p.node);

            #[allow(clippy::collapsible_if)]
            if let (Some(from_id), Some(to_id)) = (from_node_id, to_node_id) {
                if let (Some(&src_entity), Some(&target_entity)) =
                    (canvas_to_entity.get(&from_id), canvas_to_entity.get(&to_id))
                {
                    world.spawn(Edge {
                        source: src_entity,
                        target: target_entity,
                        // TODO: Map actual port names from FlowCanvas once Port struct supports names
                        source_handle: Some("Exec".to_string()),
                        target_handle: Some("Exec".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Processes pending events from the engine and updates the visual state.
    ///
    /// This is where the visualization of execution flow happens.
    pub fn sync_events(&mut self, graph: &mut GraphState<T>) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                SystemEvent::NodeTelemetry {
                    node_id, success, ..
                } => {
                    #[allow(clippy::collapsible_if)]
                    if let Some(&canvas_id) = graph.uuid_index.get(&node_id) {
                        if let Some(_node) = graph.nodes.get_mut(canvas_id) {
                            // Here we could trigger a "Pulse" animation or change style
                            // For now, let's just log it.
                            tracing::info!(node_id = ?canvas_id, success, "Node execution visualization triggered");
                        }
                    }
                }
                SystemEvent::EdgeTraversal {
                    source_id,
                    target_id,
                    ..
                } => {
                    // We could find the connection between these nodes and animate the transition
                    tracing::info!(from = ?source_id, to = ?target_id, "Edge traversal visualization triggered");
                }
                _ => {}
            }
        }
    }

    /// Runs one tick of the backend engine.
    pub async fn tick(&mut self) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine.update();
        Ok(())
    }

    /// Triggers a reload of all YAML node definitions.
    pub async fn reload_definitions(&self) -> Result<()> {
        self.api_tx
            .send(ferroflux_core::api::ApiCommand::ReloadDefinitions)
            .await?;
        Ok(())
    }

    /// Fetches all available node templates from the engine registry.
    pub async fn get_node_templates(
        &self,
    ) -> Result<Vec<ferroflux_core::traits::node_factory::NodeMetadata>> {
        let engine = self.engine.lock().await;
        // Access NodeRegistry
        let mut templates = Vec::new();

        if let Some(registry) = engine
            .world
            .get_resource::<ferroflux_core::resources::registry::NodeRegistry>()
        {
            let core_temps = registry.list_templates();
            println!("DEBUG: SDK found {} core templates", core_temps.len());
            templates.extend(core_temps);
        } else {
            println!("DEBUG: SDK: NodeRegistry resource not found!");
        }

        // Access IntegrationRegistry if available
        if let Some(registry) = engine
            .world
            .get_resource::<ferroflux_core::integrations::IntegrationRegistry>()
        {
            for (key, def) in &registry.definitions {
                for (action_key, action) in &def.actions {
                    let id = format!("integration/{}/{}", key, action_key);

                    let mut inputs = action
                        .inputs
                        .iter()
                        .map(|f| ferroflux_core::traits::node_factory::PortMetadata {
                            name: f.name.clone(),
                            data_type: "any".to_string(), // Schema types are complex, using any for now
                        })
                        .collect::<Vec<_>>();

                    // Always add Exec input
                    inputs.insert(
                        0,
                        ferroflux_core::traits::node_factory::PortMetadata {
                            name: "Exec".to_string(),
                            data_type: "flow".to_string(),
                        },
                    );

                    let mut outputs = action
                        .outputs
                        .iter()
                        .map(|f| ferroflux_core::traits::node_factory::PortMetadata {
                            name: f.name.clone(),
                            data_type: "any".to_string(),
                        })
                        .collect::<Vec<_>>();

                    // Always add Exec output
                    outputs.insert(
                        0,
                        ferroflux_core::traits::node_factory::PortMetadata {
                            name: "Success".to_string(),
                            data_type: "flow".to_string(),
                        },
                    );

                    templates.push(ferroflux_core::traits::node_factory::NodeMetadata {
                        id,
                        name: format!("{} {}", def.name, action_key),
                        category: "Integrations".to_string(),
                        platform: Some(key.clone()),
                        description: action.documentation.clone(),
                        inputs,
                        outputs,
                        settings: vec![], // For now
                    });
                }
            }
        }

        Ok(templates)
    }
}
