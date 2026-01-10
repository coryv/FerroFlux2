use crate::actor::{EngineActor, EngineCommand};
use anyhow::Result;
use ferroflux_core::api::events::SystemEvent;
use ferroflux_core::app::AppBuilder;
use flow_canvas::model::{GraphState, NodeData};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, oneshot};

pub mod actor;
pub mod reconciler;

/// The SDK Client for interacting with the FerroFlux Engine.
///
/// This client manages the lifecycle of the engine, graph deployment,
/// and event synchronization, serving as the primary interface for
/// Desktop, Web, and CLI applications.
pub struct FerroFluxClient<T: NodeData + Send + 'static> {
    /// Channel for sending commands to the Engine Actor.
    command_tx: mpsc::Sender<EngineCommand<T>>,
    /// Subscriber to the engine's event bus.
    pub event_rx: broadcast::Receiver<SystemEvent>,

    _marker: std::marker::PhantomData<T>,
}

impl<T: NodeData + Send + 'static> FerroFluxClient<T> {
    /// Initializes a new SDK client and starts the background Engine Actor.
    ///
    /// Returns the Client and the JoinHandle for the background task.
    pub async fn start() -> Result<(Self, tokio::task::JoinHandle<()>)> {
        let (mut engine, api_tx, event_tx, ..) = AppBuilder::new().build().await?;

        // Register Core Tools
        if let Some(mut registry) = engine
            .world
            .get_resource_mut::<ferroflux_core::tools::registry::ToolRegistry>()
        {
            ferroflux_core::tools::register_core_tools(&mut registry);
        }

        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let actor = EngineActor::new(engine, api_tx.clone(), cmd_rx, event_tx.clone());

        let handle = tokio::spawn(actor.run());

        let client = Self {
            command_tx: cmd_tx,
            event_rx: event_tx.subscribe(),
            _marker: std::marker::PhantomData,
        };

        Ok((client, handle))
    }

    /// Synchronizes the visual graph state with the running engine.
    ///
    /// This uses incremental reconciliation to preserve runtime state of existing nodes.
    pub async fn sync_graph(&self, graph: &GraphState<T>) -> Result<()> {
        // Clone the graph to send to the actor
        let graph_clone = Box::new(graph.clone());
        self.command_tx
            .send(EngineCommand::SyncGraph(graph_clone))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;
        Ok(())
    }

    /// Pauses engine execution.
    pub async fn pause(&self) -> Result<()> {
        self.command_tx
            .send(EngineCommand::Pause)
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;
        Ok(())
    }

    /// Resumes engine execution.
    pub async fn resume(&self) -> Result<()> {
        self.command_tx
            .send(EngineCommand::Resume)
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;
        Ok(())
    }

    /// Executes a single tick (if paused).
    pub async fn step(&self, count: usize) -> Result<()> {
        self.command_tx
            .send(EngineCommand::Step(count))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;
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
                } =>
                {
                    #[allow(clippy::collapsible_if)]
                    if let Some(&canvas_id) = graph.uuid_index.get(&node_id) {
                        if let Some(_node) = graph.nodes.get_mut(canvas_id) {
                            tracing::info!(node_id = ?canvas_id, success, "Node execution visualization triggered");
                        }
                    }
                }
                SystemEvent::EdgeTraversal {
                    source_id,
                    target_id,
                    ..
                } => {
                    tracing::info!(from = ?source_id, to = ?target_id, "Edge traversal visualization triggered");
                }
                _ => {}
            }
        }
    }

    /// Simulates a node execution in Shadow Mode and waits for the result.
    pub async fn simulate_and_wait(
        &mut self,
        node_id: uuid::Uuid,
        definition_id: String,
        config: HashMap<String, serde_json::Value>,
        input_payload: serde_json::Value,
        mock_config: HashMap<String, ferroflux_core::components::shadow::MockConfig>,
    ) -> Result<serde_json::Value> {
        let trace_id = format!("sim-{}", uuid::Uuid::new_v4());
        let tenant_id = ferroflux_iam::TenantId::from("default");

        // Subscribe to events *before* sending command to avoid race
        let mut rx = self.event_rx.resubscribe();

        self.command_tx
            .send(EngineCommand::Api(
                ferroflux_core::api::ApiCommand::SimulateNode {
                    tenant_id,
                    node_id,
                    definition_id,
                    config,
                    input_payload,
                    trace_id: trace_id.clone(),
                    mock_config,
                },
            ))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;

        // Wait for result
        // TODO: Add timeout
        let start = std::time::Instant::now();
        loop {
            // We do NOT tick here anymore. The actor ticks.

            while let Ok(event) = rx.try_recv() {
                match event {
                    SystemEvent::NodeTelemetry {
                        trace_id: t_id,
                        details,
                        ..
                    } if t_id == trace_id => {
                        return Ok(details);
                    }
                    SystemEvent::NodeError {
                        trace_id: t_id,
                        error,
                        ..
                    } if t_id == trace_id => {
                        return Err(anyhow::anyhow!("Simulation failed: {}", error));
                    }
                    _ => {}
                }
            }

            if start.elapsed().as_secs() > 5 {
                return Err(anyhow::anyhow!("Simulation timed out"));
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    /// Fetches all available node templates.
    ///
    /// NOTE: This now requires a request-response pattern because we don't own the App Mutex.
    /// For V1, we simply can't access templates directly if the Actor owns the App.
    ///
    /// SOLUTION: Implementation Phase 2 should add a `GetTemplates` command + Response channel.
    /// For now, keeping as TODO or removing.
    /// If the Playground needs this, it won't work with this refactor immediately unless we add it to EngineCommand.
    ///
    /// Let's add a placeholder comment.
    /// Triggers a reload of all YAML node definitions.
    pub async fn reload_definitions(&self) -> Result<()> {
        self.command_tx
            .send(EngineCommand::Api(
                ferroflux_core::api::ApiCommand::ReloadDefinitions,
            ))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;
        Ok(())
    }

    /// Fetches all available node templates.
    pub async fn get_node_templates(
        &self,
    ) -> Result<Vec<ferroflux_core::traits::node_factory::NodeMetadata>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(EngineCommand::GetTemplates(tx))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;

        match rx.await {
            Ok(res) => res.map_err(|e| anyhow::anyhow!("Failed to list templates: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Response channel closed")),
        }
    }
}
