use crate::persistence::{NodeRuntimeState, RuntimeSnapshot};
use crate::reconciler::reconcile_graph;
use bevy_ecs::prelude::*;
use ferroflux_core::components::core::{Inbox, NodeConfig, Outbox};

use ferroflux_core::api::ApiCommand;
use ferroflux_core::app::App;
use flow_canvas::model::{GraphState, NodeData};
use tokio::sync::{broadcast, mpsc, oneshot};

/// Commands sent to the Engine Actor.
pub enum EngineCommand<T: NodeData + Send + 'static> {
    /// Relay a standard core API command.
    Api(ApiCommand),
    /// Reconcile the running world with the new visual graph state.
    SyncGraph(Box<GraphState<T>>),
    /// Pause execution (continue processing commands, but skip `app.update()`).
    Pause,
    /// Resume execution.
    Resume,
    /// Execute a single tick and then pause.
    Step(usize),
    /// Fetch available node templates.
    GetTemplates(
        oneshot::Sender<Result<Vec<ferroflux_core::traits::node_factory::NodeMetadata>, String>>,
    ),
    /// Save a snapshot of the runtime state.
    SaveSnapshot(oneshot::Sender<Result<RuntimeSnapshot, String>>),
    /// Restore a snapshot of the runtime state.
    RestoreSnapshot(RuntimeSnapshot),
    /// Inspect the internal state of a specific node.
    InspectNode(uuid::Uuid, oneshot::Sender<Option<NodeRuntimeState>>),
    /// Inject a message into a node's inbox.
    InjectMessage {
        node_id: uuid::Uuid,
        port: String,
        payload: serde_json::Value,
    },
    /// Read a blob from the store.
    ReadBlob(
        ferroflux_core::store::SecureTicket,
        oneshot::Sender<Option<serde_json::Value>>,
    ),
}

/// The Actor that owns the App and runs the main loop.
pub struct EngineActor<T: NodeData + Send + 'static> {
    app: App,
    api_tx: async_channel::Sender<ApiCommand>, // To send commands to internal worker
    command_rx: mpsc::Receiver<EngineCommand<T>>,
    #[allow(dead_code)]
    event_tx: broadcast::Sender<ferroflux_core::api::events::SystemEvent>,

    // State
    paused: bool,
    steps_remaining: usize,
}

impl<T: NodeData + Send + 'static> EngineActor<T> {
    pub fn new(
        mut app: App,
        api_tx: async_channel::Sender<ApiCommand>,
        command_rx: mpsc::Receiver<EngineCommand<T>>,
        event_tx: broadcast::Sender<ferroflux_core::api::events::SystemEvent>,
    ) -> Self {
        // Inject Event Bus so Core systems can publish events
        app.world
            .insert_resource(ferroflux_core::api::events::SystemEventBus(
                event_tx.clone(),
            ));

        Self {
            app,
            api_tx,
            command_rx,
            event_tx,
            paused: false,
            steps_remaining: 0,
        }
    }

    pub async fn run(mut self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16)); // ~60Hz target

        loop {
            // Priority: Check commands
            tokio::select! {
                // 1. Handle Commands
                cmd_opt = self.command_rx.recv() => {
                    match cmd_opt {
                        Some(cmd) => self.handle_command(cmd).await,
                        None => {
                            tracing::info!("EngineActor: Command channel closed. Shutting down.");
                            break;
                        }
                    }
                }

                // 2. Tick (if not paused)
                _ = interval.tick() => {
                    if !self.paused || self.steps_remaining > 0 {
                        self.app.update();

                        // Decrement step if stepping
                        if self.steps_remaining > 0 {
                             self.steps_remaining -= 1;
                        }
                    }
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: EngineCommand<T>) {
        match cmd {
            EngineCommand::Api(api_cmd) => {
                // Forward to internal API worker
                if let Err(e) = self.api_tx.send(api_cmd).await {
                    tracing::error!("EngineActor: Failed to forward ApiCommand: {}", e);
                }
            }
            EngineCommand::SyncGraph(graph) => {
                if let Err(e) = reconcile_graph(&mut self.app.world, &*graph) {
                    tracing::error!("EngineActor: Failed to reconcile graph: {}", e);
                }
            }
            EngineCommand::Pause => {
                self.paused = true;
                tracing::info!("EngineActor: Paused");
            }
            EngineCommand::Resume => {
                self.paused = false;
                tracing::info!("EngineActor: Resumed");
            }
            EngineCommand::Step(n) => {
                if self.paused {
                    self.steps_remaining = n;
                }
            }
            EngineCommand::GetTemplates(tx) => {
                let templates = self.get_node_templates();
                let _ = tx.send(templates);
            }
            EngineCommand::SaveSnapshot(tx) => {
                let snapshot = self.save_runtime_snapshot();
                let _ = tx.send(snapshot);
            }
            EngineCommand::RestoreSnapshot(snapshot) => {
                if let Err(e) = self.restore_runtime_snapshot(snapshot) {
                    tracing::error!("EngineActor: Failed to restore snapshot: {}", e);
                }
            }
            EngineCommand::InspectNode(node_id, tx) => {
                let state = self.inspect_node(node_id);
                let _ = tx.send(state);
            }
            EngineCommand::InjectMessage {
                node_id,
                port,
                payload,
            } => {
                if let Err(e) = self.inject_message(node_id, port, payload) {
                    tracing::error!("EngineActor: Failed to inject message: {}", e);
                }
            }
            EngineCommand::ReadBlob(ticket, tx) => {
                let value = self.read_blob(ticket);
                let _ = tx.send(value);
            }
        }
    }

    fn read_blob(
        &mut self,
        ticket: ferroflux_core::store::SecureTicket,
    ) -> Option<serde_json::Value> {
        let world = &mut self.app.world;
        if let Some(store) = world.get_resource::<ferroflux_core::store::BlobStore>() {
            return store
                .claim(&ticket)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok());
        }
        None
    }

    fn inject_message(
        &mut self,
        node_id: uuid::Uuid,
        _port: String,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let world = &mut self.app.world;

        // 1. Store Payload in BlobStore
        let ticket = if let Some(store) = world.get_resource::<ferroflux_core::store::BlobStore>() {
            let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
            store.check_in(&bytes).map_err(|e| e.to_string())?
        } else {
            return Err("BlobStore not found".to_string());
        };

        // 2. Find Node and Update Inbox
        let mut query = world.query::<(&NodeConfig, &mut Inbox)>();
        for (config, mut inbox) in query.iter_mut(world) {
            if config.id == node_id {
                inbox.queue.push_back((None, ticket));
                return Ok(());
            }
        }
        Err(format!("Node {} not found", node_id))
    }

    fn inspect_node(&mut self, node_id: uuid::Uuid) -> Option<NodeRuntimeState> {
        let world = &mut self.app.world;
        let mut query = world.query::<(&NodeConfig, &Inbox, &Outbox)>();

        for (config, inbox, outbox) in query.iter(world) {
            if config.id == node_id {
                return Some(NodeRuntimeState {
                    inbox: inbox.clone(),
                    outbox: outbox.clone(),
                });
            }
        }
        None
    }

    fn save_runtime_snapshot(&mut self) -> Result<RuntimeSnapshot, String> {
        let world = &mut self.app.world;
        let mut node_states = std::collections::HashMap::new();

        let mut query = world.query::<(&NodeConfig, &Inbox, &Outbox)>();
        for (config, inbox, outbox) in query.iter(world) {
            node_states.insert(
                config.id,
                NodeRuntimeState {
                    inbox: inbox.clone(),
                    outbox: outbox.clone(),
                },
            );
        }

        // Snapshot BlobStore
        let blobs = if let Some(store) = world.get_resource::<ferroflux_core::store::BlobStore>() {
            store.snapshot()
        } else {
            Vec::new()
        };

        Ok(RuntimeSnapshot { node_states, blobs })
    }

    fn restore_runtime_snapshot(&mut self, snapshot: RuntimeSnapshot) -> Result<(), String> {
        let world = &mut self.app.world;

        // Restore Blobs
        if let Some(store) = world.get_resource::<ferroflux_core::store::BlobStore>() {
            store.restore(snapshot.blobs).map_err(|e| e.to_string())?;
        }

        // Restore Node States
        // We have to iterate all entities with NodeConfig, match ID, and update components.
        let mut query = world.query::<(Entity, &NodeConfig)>();
        let mut target_entities = Vec::new();

        for (entity, config) in query.iter(world) {
            if let Some(state) = snapshot.node_states.get(&config.id) {
                target_entities.push((entity, state.clone()));
            }
        }

        for (entity, state) in target_entities {
            // We use insert to overwrite or add components
            world.entity_mut(entity).insert((state.inbox, state.outbox));
        }

        Ok(())
    }

    // Internal helper to fetch templates from the owned World
    fn get_node_templates(
        &self,
    ) -> Result<Vec<ferroflux_core::traits::node_factory::NodeMetadata>, String> {
        let mut templates = Vec::new();
        let world = &self.app.world;

        // Access NodeRegistry
        if let Some(registry) =
            world.get_resource::<ferroflux_core::resources::NodeRegistry>()
        {
            templates.extend(registry.list_templates());
        } else {
            tracing::warn!("EngineActor: NodeRegistry resource not found!");
        }

        // Access IntegrationRegistry if available
        if let Some(registry) =
            world.get_resource::<ferroflux_core::integrations::IntegrationRegistry>()
        {
            for (key, def) in &registry.definitions {
                for (action_key, action) in &def.actions {
                    let id = format!("integration/{}/{}", key, action_key);

                    let mut inputs = action
                        .inputs
                        .iter()
                        .map(|f| ferroflux_core::traits::node_factory::PortMetadata {
                            name: f.name.clone(),
                            data_type: "any".to_string(),
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
