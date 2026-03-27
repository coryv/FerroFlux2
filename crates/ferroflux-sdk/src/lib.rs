use crate::actor::{EngineActor, EngineCommand};
use anyhow::Result;
use ferroflux_core::api::events::SystemEvent;
use ferroflux_core::app::AppBuilder;
use flow_canvas::model::{GraphState, NodeData};
use futures::StreamExt;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_stream::wrappers::BroadcastStream;

pub mod actor;
pub mod persistence;
pub mod protocol;
pub mod reconciler;
pub mod testing;

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

use serde::Serialize;
use serde::de::DeserializeOwned;

impl<T: NodeData + Serialize + DeserializeOwned + Send + 'static> FerroFluxClient<T> {
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

        // Wait for result (5-second timeout)
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

    /// Subscribes to a stream of application logs.
    pub fn logs(&self) -> impl futures::Stream<Item = (String, String)> + Send + 'static {
        BroadcastStream::new(self.event_rx.resubscribe()).filter_map(|res| async move {
            if let Ok(SystemEvent::Log { level, message, .. }) = res {
                Some((level, message))
            } else {
                None
            }
        })
    }

    /// Subscribes to a stream of node telemetry events (execution results).
    pub fn telemetry(
        &self,
    ) -> impl futures::Stream<Item = (uuid::Uuid, bool, u64, String)> + Send + 'static {
        BroadcastStream::new(self.event_rx.resubscribe()).filter_map(|res| async move {
            if let Ok(SystemEvent::NodeTelemetry {
                node_id,
                success,
                execution_ms,
                trace_id,
                ..
            }) = res
            {
                Some((node_id, success, execution_ms, trace_id))
            } else {
                None
            }
        })
    }

    /// Captures the full state of the engine (Structure + Runtime Data) into a JSON string.
    pub async fn save_snapshot(&self, graph: &GraphState<T>) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(EngineCommand::SaveSnapshot(tx))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;

        let runtime = rx
            .await
            .map_err(|_| anyhow::anyhow!("Response channel closed"))?
            .map_err(|e| anyhow::anyhow!("Snapshot failed: {}", e))?;

        let save_file = crate::persistence::SaveFile {
            graph: graph.clone(),
            runtime,
        };

        Ok(serde_json::to_string(&save_file)?)
    }

    /// Restores the engine state from a JSON snapshot.
    ///
    /// This will sync the graph structure and then restore the runtime queues/memory.
    pub async fn load_snapshot(&self, json: &str) -> Result<()> {
        let save_file: crate::persistence::SaveFile<T> = serde_json::from_str(json)?;

        // 1. Restore Structure
        self.sync_graph(&save_file.graph).await?;

        // 2. Restore Runtime State
        self.command_tx
            .send(EngineCommand::RestoreSnapshot(save_file.runtime))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;

        Ok(())
    }

    /// Inspects the runtime state (Inbox, Outbox) of a specific node.
    pub async fn inspect_node(
        &self,
        node_id: uuid::Uuid,
    ) -> Result<Option<crate::persistence::NodeRuntimeState>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(EngineCommand::InspectNode(node_id, tx))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;

        let state = rx
            .await
            .map_err(|_| anyhow::anyhow!("Response channel closed"))?;
        Ok(state)
    }

    /// Injects a message into a node's input.
    pub async fn inject_message(
        &self,
        node_id: uuid::Uuid,
        port: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<()> {
        self.command_tx
            .send(EngineCommand::InjectMessage {
                node_id,
                port: port.into(),
                payload,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;
        Ok(())
    }

    /// Reads a blob from the store given a ticket.
    pub async fn read_blob(
        &self,
        ticket: ferroflux_core::store::SecureTicket,
    ) -> Result<Option<serde_json::Value>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(EngineCommand::ReadBlob(ticket, tx))
            .await
            .map_err(|_| anyhow::anyhow!("Engine Actor closed"))?;

        let val = rx
            .await
            .map_err(|_| anyhow::anyhow!("Response channel closed"))?;
        Ok(val)
    }
}
