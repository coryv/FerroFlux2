use crate::reconciler::reconcile_graph;

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
        app: App,
        api_tx: async_channel::Sender<ApiCommand>,
        command_rx: mpsc::Receiver<EngineCommand<T>>,
        event_tx: broadcast::Sender<ferroflux_core::api::events::SystemEvent>,
    ) -> Self {
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
        }
    }

    // Internal helper to fetch templates from the owned World
    fn get_node_templates(
        &self,
    ) -> Result<Vec<ferroflux_core::traits::node_factory::NodeMetadata>, String> {
        let mut templates = Vec::new();
        let world = &self.app.world;

        // Access NodeRegistry
        if let Some(registry) =
            world.get_resource::<ferroflux_core::resources::registry::NodeRegistry>()
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
