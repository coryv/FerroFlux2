use anyhow::Result;
use bevy_ecs::prelude::*;
use ferroflux_core::api::events::SystemEvent;
use ferroflux_core::app::App;
use ferroflux_core::components::core::{Edge, NodeConfig};
use flow_canvas::model::{GraphState, NodeData, NodeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

/// The Bridge between the visual FlowCanvas and the FerroFlux computation engine.
///
/// It manages the lifecycle of the engine and the synchronization of data
/// without coupling the two core domains directly.
pub struct FerroFluxAdapter<T: NodeData> {
    /// Handle to the underlying FerroFlux engine.
    pub engine: Arc<Mutex<App>>,
    /// Mapping from stable Engine UUIDs to transient Canvas Node IDs.
    pub uuid_to_canvas: HashMap<Uuid, NodeId>,
    /// Mapping from transient Canvas Node IDs to stable Engine UUIDs.
    pub canvas_to_uuid: HashMap<NodeId, Uuid>,
    /// Subscriber to the engine's event bus.
    event_rx: broadcast::Receiver<SystemEvent>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: NodeData> FerroFluxAdapter<T> {
    /// Creates a new adapter for a given engine instance.
    pub fn new(engine: App, event_bus: broadcast::Sender<SystemEvent>) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            uuid_to_canvas: HashMap::new(),
            canvas_to_uuid: HashMap::new(),
            event_rx: event_bus.subscribe(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Deploys the current Canvas state to the Engine.
    ///
    /// This "lowers" the visual graph into a set of ECS entities and components.
    /// It strips away layout information (position, size) as the engine doesn't need it.
    pub async fn deploy(&mut self, graph: &GraphState<T>) -> Result<()> {
        let mut engine = self.engine.lock().await;
        let world = &mut engine.world;

        // 1. Clear existing nodes/edges from the engine (simplified for V1)
        // In a real app, we might want incremental updates.
        let mut query = world.query_filtered::<Entity, With<NodeConfig>>();
        let entities: Vec<Entity> = query.iter(world).collect();
        for entity in entities {
            world.despawn(entity);
        }

        self.uuid_to_canvas.clear();
        self.canvas_to_uuid.clear();

        let mut canvas_to_entity = HashMap::new();

        // 2. Spawn Nodes
        for (id, node) in &graph.nodes {
            let entity = world
                .spawn(NodeConfig {
                    id: node.uuid,
                    name: format!("{:?}", node.id), // Placeholder name
                    node_type: "Default".to_string(), // This would come from node.data in a real app
                    workflow_id: None,
                    tenant_id: None,
                })
                .id();

            canvas_to_entity.insert(id, entity);
            self.uuid_to_canvas.insert(node.uuid, id);
            self.canvas_to_uuid.insert(id, node.uuid);
        }

        // 3. Spawn Edges
        for (_, conn) in &graph.connections {
            let from_node_id = graph.ports.get(conn.from).map(|p| p.node);
            let to_node_id = graph.ports.get(conn.to).map(|p| p.node);

            if let (Some(from_id), Some(to_id)) = (from_node_id, to_node_id) {
                if let (Some(&src_entity), Some(&target_entity)) =
                    (canvas_to_entity.get(&from_id), canvas_to_entity.get(&to_id))
                {
                    world.spawn(Edge {
                        source: src_entity,
                        target: target_entity,
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
                    if let Some(&canvas_id) = self.uuid_to_canvas.get(&node_id) {
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
}
