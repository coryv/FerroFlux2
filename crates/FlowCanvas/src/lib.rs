//! # FlowCanvas
//!
//! `flow_canvas` is a headless, highly optimized, retained-mode graph library.
//! It is designed to be the "operating system" for node graphs, handling state,
//! mathematics, and logic, while delegating rendering to the host application.
//!
//! ## Core Architecture
//! - **Model (`src/model.rs`)**: Stores the graph state in a flat arena (SlotMap).
//! - **View (`src/view.rs`)**: Handles coordinate transformation (World <-> Screen).
//! - **Render (`src/render.rs`)**: Outputs a list of `DrawCommand`s for the host to render.

pub mod config;
pub mod history;
pub mod input;
pub mod interaction;
pub mod math;
pub mod model;
pub mod painter;
pub mod persistence;
pub mod render;
pub mod view;

use glam::Vec2;
use input::InputState;
use model::GraphState;
use render::RenderList;
use view::{Transform, View};

// Re-exports for convenience
pub use config::CanvasConfig;
pub use interaction::{InteractionMode, LogicEvent};

/// The main entry point for the library.
///
/// The `Canvas` struct holds the transient state of the editor (viewport, input state)
/// and generic configuration. It is intended to be instantiated once and reused.
pub struct Canvas {
    /// Configuration settings.
    pub config: CanvasConfig,
    /// The Viewport system handling coordinate transforms.
    pub view: View,
    /// Current interaction mode.
    pub interaction_mode: InteractionMode,
}

impl Canvas {
    /// Creates a new Canvas instance with the given configuration.
    pub fn new(config: CanvasConfig) -> Self {
        Self {
            config,
            view: View::new(Transform::default(), Vec2::new(800.0, 600.0)), // Default 800x600, user should update
            interaction_mode: InteractionMode::Idle,
        }
    }

    /// Updates the viewport size (e.g., on window resize).
    ///
    /// This should be called whenever the host application's window or panel size changes.
    pub fn update_viewport_size(&mut self, size: Vec2) {
        self.view.viewport_size = size;
    }

    /// The core update loop.
    ///
    /// This function should be called every frame (or on event). It processes the `GraphState`
    /// and returns a list of drawing commands (`RenderList`) that the host application should render.
    pub fn update<T: model::NodeData>(
        &mut self,
        input: &InputState,
        _dt: f32,
        graph: &mut GraphState<T>,
    ) -> (RenderList, Vec<LogicEvent>) {
        let mut logic_events = Vec::new();

        // 1. Handle Interactions (Pan, Zoom, Select, Drag)
        interaction::handle_interactions(
            &mut self.interaction_mode,
            &mut self.view,
            &self.config,
            input,
            graph,
            &mut logic_events,
        );

        // 2. Render
        let draw_list = painter::Painter::draw_graph(
            &self.view,
            &self.config,
            graph,
            &self.interaction_mode,
            input.screen_size,
        );

        (draw_list, logic_events)
    }
}
