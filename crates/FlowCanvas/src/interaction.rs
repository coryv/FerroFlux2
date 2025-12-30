use glam::Vec2;
use std::collections::HashMap;

use crate::config::CanvasConfig;
use crate::input::{self, InputState};
use crate::model::{self, GraphState, NodeFlags, NodeId};
use crate::view::{Transform, View};

/// Events emitted by the Canvas logic to the host application.
#[derive(Clone, Debug, PartialEq)]
pub enum LogicEvent {
    /// Request to connect two ports.
    Connect {
        from: model::PortId,
        to: model::PortId,
    },
    /// Request to delete selected nodes.
    DeleteSelection,
    /// A selection of nodes was moved.
    NodesMoved {
        ids: Vec<model::NodeId>,
        /// The delta applied in World Space.
        delta: Vec2,
    },
    /// The graph visual state has changed, requiring a repaint.
    /// This is useful for power efficiency (e.g., only render when dirty).
    RepaintNeeded,
}

/// The current state of user interaction.
#[derive(Clone, Debug)]
pub enum InteractionMode {
    /// No active interaction.
    Idle,
    /// User is panning the canvas (Middle click / Space+Drag).
    Panning {
        /// Mouse position at start of drag (Screen Space).
        start_drag: Vec2,
        /// Transform at start of drag.
        initial_transform: Transform,
    },
    /// User is moving a selection of nodes.
    DraggingNodes {
        /// The list of nodes being dragged.
        nodes: Vec<NodeId>,
        /// Initial positions of the nodes when drag started (World Space).
        initial_positions: HashMap<NodeId, Vec2>,
        /// Mouse position when drag started (World Space).
        start_mouse_world: Vec2,
    },
    /// User is creating a connection.
    Linking {
        /// The port where the wire started.
        source: model::PortId,
        /// Current temporary endpoint of the wire (World Space).
        curr_pos_world: Vec2,
    },
    /// User is box selecting.
    BoxSelecting {
        /// Start of the selection box (World Space).
        start_pos_world: Vec2,
        /// Current end of the selection box (World Space).
        current_pos_world: Vec2,
    },
}

/// Handles user interactions and updates the graph/view state.
///
/// This function acts as the central state machine for the Canvas. It processes
/// input events based on the current `InteractionMode` and transitions between states.
/// It also emits `LogicEvent`s when significant actions occur (e.g., connection created).
///
/// # Arguments
/// * `mode` - The current interaction mode (will be mutated on state transitions).
/// * `view` - The viewport state (pan/zoom), mutated during panning/zooming.
/// * `config` - Configuration settings (e.g., snap threshold).
/// * `input` - The input state for the current frame.
/// * `graph` - The graph data, mutated during selection/dragging.
/// * `_events` - A buffer to push `LogicEvent`s into.
pub fn handle_interactions<T: model::NodeData>(
    mode: &mut InteractionMode,
    view: &mut View,
    config: &CanvasConfig,
    input: &InputState,
    graph: &mut GraphState<T>,
    _events: &mut Vec<LogicEvent>,
) {
    // Zooming via Scroll
    // Zooming via Scroll
    if input.scroll_delta != 0.0 {
        let old_zoom = view.transform.zoom;
        let zoom_factor = 1.0 + (input.scroll_delta * config.zoom_speed);
        let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 10.0);

        if (new_zoom - old_zoom).abs() > f32::EPSILON {
            // 1. Calculate world position under mouse BEFORE zoom
            let world_mouse = view.screen_to_world(input.mouse_pos);

            // 2. Apply new zoom
            view.transform.zoom = new_zoom;

            // 3. Adjust Pan so that world_mouse remains under input.mouse_pos
            // Screen = World * Zoom + Pan  =>  Pan = Screen - (World * Zoom)
            view.transform.pan = input.mouse_pos - (world_mouse * new_zoom);

            _events.push(LogicEvent::RepaintNeeded);
        }
    }

    // Keyboard Shortcuts
    if !input.event_consumed_by_content {
        for key in &input.pressed_keys {
            match key {
                input::Key::Delete | input::Key::Backspace => {
                    _events.push(LogicEvent::DeleteSelection);
                    _events.push(LogicEvent::RepaintNeeded);
                }
                input::Key::A => {
                    if input.modifiers.ctrl || input.modifiers.meta {
                        // Select All
                        for (_, node) in &mut graph.nodes {
                            node.flags.insert(NodeFlags::SELECTED);
                        }
                        _events.push(LogicEvent::RepaintNeeded);
                    }
                }
            }
        }
    }

    let next_mode = match mode {
        InteractionMode::Idle => handle_idle(view, input, graph, _events),
        InteractionMode::Panning {
            start_drag,
            initial_transform,
        } => handle_panning(view, input, *start_drag, *initial_transform, _events),
        InteractionMode::DraggingNodes {
            nodes,
            initial_positions,
            start_mouse_world,
        } => handle_dragging_nodes(
            view,
            input,
            graph,
            nodes,
            initial_positions,
            *start_mouse_world,
            _events,
        ),
        InteractionMode::Linking {
            source,
            curr_pos_world,
        } => handle_linking(view, config, input, graph, *source, curr_pos_world, _events),
        InteractionMode::BoxSelecting {
            start_pos_world,
            current_pos_world,
        } => handle_box_selecting(
            view,
            input,
            graph,
            *start_pos_world,
            current_pos_world,
            _events,
        ),
    };

    if let Some(new_mode) = next_mode {
        *mode = new_mode;
    }
}

/// Handles the `Idle` state interactions.
///
/// This checks for inputs to transition into:
/// - `Panning` (middle click)
/// - `Linking` (clicking a port)
/// - `DraggingNodes` (clicking a node)
/// - `BoxSelecting` (clicking empty space)
fn handle_idle<T: model::NodeData>(
    view: &View,
    input: &InputState,
    graph: &mut GraphState<T>,
    _events: &mut Vec<LogicEvent>,
) -> Option<InteractionMode> {
    // Transition to Panning
    if input.mouse_buttons.middle && !input.event_consumed_by_content {
        return Some(InteractionMode::Panning {
            start_drag: input.mouse_pos,
            initial_transform: view.transform,
        });
    } else if input.mouse_buttons.left && !input.event_consumed_by_content {
        let world_mouse = view.screen_to_world(input.mouse_pos);
        let mut hit_port = None;
        let mut hit_node = None;

        // Hit Test Ports FIRST (Priority)
        // Iterate front to back
        'port_search: for &node_id in graph.draw_order.iter().rev() {
            if let Some(node) = graph.nodes.get(node_id) {
                // Check inputs
                let spacing_in = node.size.y / (node.inputs.len() as f32 + 1.0);
                for (i, &port_id) in node.inputs.iter().enumerate() {
                    let local_y = spacing_in * (i as f32 + 1.0);
                    let port_pos = node.position + Vec2::new(0.0, local_y);
                    if port_pos.distance(world_mouse) <= (10.0 / view.transform.zoom).max(5.0) {
                        hit_port = Some(port_id);
                        break 'port_search;
                    }
                }

                // Check outputs
                let spacing_out = node.size.y / (node.outputs.len() as f32 + 1.0);
                for (i, &port_id) in node.outputs.iter().enumerate() {
                    let local_y = spacing_out * (i as f32 + 1.0);
                    let port_pos = node.position + Vec2::new(node.size.x, local_y);
                    if port_pos.distance(world_mouse) <= (10.0 / view.transform.zoom).max(5.0) {
                        hit_port = Some(port_id);
                        break 'port_search;
                    }
                }
            }
        }

        if let Some(port_id) = hit_port {
            // Start Linking
            return Some(InteractionMode::Linking {
                source: port_id,
                curr_pos_world: world_mouse,
            });
        }

        // Hit test Nodes interaction
        // Iterate in reverse draw order (front to back)
        for &node_id in graph.draw_order.iter().rev() {
            if let Some(node) = graph.nodes.get(node_id)
                && world_mouse.x >= node.position.x
                && world_mouse.x <= node.position.x + node.size.x
                && world_mouse.y >= node.position.y
                && world_mouse.y <= node.position.y + node.size.y
            {
                hit_node = Some(node_id);
                break;
            }
        }

        if let Some(node_id) = hit_node {
            // Selection Logic
            if !input.modifiers.shift {
                // Deselect others
                for (_, node) in &mut graph.nodes {
                    node.flags.remove(NodeFlags::SELECTED);
                }
            }

            // Select this one
            if let Some(node) = graph.nodes.get_mut(node_id) {
                node.flags.insert(NodeFlags::SELECTED);
            }

            // Bring to front
            graph.draw_order.retain(|&id| id != node_id);
            graph.draw_order.push(node_id);
            _events.push(LogicEvent::RepaintNeeded);

            // Transition to Dragging
            let mut initial_positions = HashMap::new();
            let mut selected_nodes = Vec::new();
            for (id, node) in &graph.nodes {
                if node.flags.contains(NodeFlags::SELECTED) {
                    selected_nodes.push(id);
                    initial_positions.insert(id, node.position);
                }
            }

            return Some(InteractionMode::DraggingNodes {
                nodes: selected_nodes,
                initial_positions,
                start_mouse_world: world_mouse,
            });
        } else {
            // Clicked on empty space -> Deselect all (unless shift?)
            if !input.modifiers.shift {
                for (_, node) in &mut graph.nodes {
                    node.flags.remove(NodeFlags::SELECTED);
                }
            }

            // Start Box Selecting
            return Some(InteractionMode::BoxSelecting {
                start_pos_world: world_mouse,
                current_pos_world: world_mouse,
            });
        }
    }
    None
}

/// Handles the `Panning` state interactions.
///
/// Updates the view's pan offset based on mouse delta.
/// Returns to `Idle` on mouse release.
fn handle_panning(
    view: &mut View,
    input: &InputState,
    start_drag: Vec2,
    initial_transform: Transform,
    _events: &mut Vec<LogicEvent>,
) -> Option<InteractionMode> {
    if !input.mouse_buttons.middle {
        Some(InteractionMode::Idle)
    } else {
        let delta = input.mouse_pos - start_drag;
        view.transform.pan = initial_transform.pan + delta;
        _events.push(LogicEvent::RepaintNeeded);
        None
    }
}

/// Handles the `DraggingNodes` state interactions.
///
/// Updates the position of all selected nodes based on mouse delta.
/// Returns to `Idle` on mouse release.
#[allow(clippy::too_many_arguments)]
fn handle_dragging_nodes<T: model::NodeData>(
    view: &View,
    input: &InputState,
    graph: &mut GraphState<T>,
    nodes: &[NodeId],
    initial_positions: &HashMap<NodeId, Vec2>,
    start_mouse_world: Vec2,
    _events: &mut Vec<LogicEvent>,
) -> Option<InteractionMode> {
    if !input.mouse_buttons.left {
        // Emit event on release?
        Some(InteractionMode::Idle)
    } else {
        let current_mouse_world = view.screen_to_world(input.mouse_pos);
        let delta = current_mouse_world - start_mouse_world;

        for node_id in nodes.iter() {
            if let Some(initial_pos) = initial_positions.get(node_id)
                && let Some(node) = graph.nodes.get_mut(*node_id)
            {
                node.position = *initial_pos + delta;
            }
        }
        _events.push(LogicEvent::RepaintNeeded);
        None
    }
}

/// Handles the `Linking` state interactions.
///
/// Updates the temporary wire position and handles snapping to valid ports.
/// Emits `LogicEvent::Connect` on release over a valid target.
/// Returns to `Idle` on mouse release.
#[allow(clippy::too_many_arguments)]
fn handle_linking<T: model::NodeData>(
    view: &View,
    config: &CanvasConfig,
    input: &InputState,
    graph: &GraphState<T>,
    source: model::PortId,
    curr_pos_world: &mut Vec2,
    _events: &mut Vec<LogicEvent>,
) -> Option<InteractionMode> {
    let world_mouse = view.screen_to_world(input.mouse_pos);
    *curr_pos_world = world_mouse; // Update wire
    _events.push(LogicEvent::RepaintNeeded);

    // Snap
    // Find closest port within snap threshold
    let mut closest_dist = config.snap_threshold / view.transform.zoom; // Logic threshold
    let mut snap_target = None;

    // This is O(Ports), naive but fine for V1
    for (port_id, _port) in &graph.ports {
        if port_id == source {
            continue;
        } // Don't snap to self

        if let Some(pos) = graph.find_port_position(port_id) {
            let dist = pos.distance(world_mouse);
            if dist < closest_dist {
                closest_dist = dist;
                snap_target = Some(port_id);
            }
        }
    }

    if let Some(target) = snap_target
        && let Some(pos) = graph.find_port_position(target)
    {
        *curr_pos_world = pos; // Snap visual
    }

    if !input.mouse_buttons.left {
        // Release
        if let Some(target) = snap_target {
            _events.push(LogicEvent::Connect {
                from: source,
                to: target,
            });
            _events.push(LogicEvent::RepaintNeeded);
        }
        return Some(InteractionMode::Idle);
    }
    None
}

/// Handles the `BoxSelecting` state interactions.
///
/// Updates the selection box and selects nodes overlapping with it.
/// Returns to `Idle` on mouse release.
fn handle_box_selecting<T: model::NodeData>(
    view: &View,
    input: &InputState,
    graph: &mut GraphState<T>,
    start_pos_world: Vec2,
    current_pos_world: &mut Vec2,
    _events: &mut Vec<LogicEvent>,
) -> Option<InteractionMode> {
    let world_mouse = view.screen_to_world(input.mouse_pos);
    *current_pos_world = world_mouse;
    _events.push(LogicEvent::RepaintNeeded);

    if !input.mouse_buttons.left {
        // 1. Calculate AABB in world space
        let min_x = start_pos_world.x.min(current_pos_world.x);
        let max_x = start_pos_world.x.max(current_pos_world.x);
        let min_y = start_pos_world.y.min(current_pos_world.y);
        let max_y = start_pos_world.y.max(current_pos_world.y);

        // 2. Select nodes within
        if !input.modifiers.shift {
            for (_, node) in &mut graph.nodes {
                node.flags.remove(NodeFlags::SELECTED);
            }
        }

        for (_, node) in &mut graph.nodes {
            // Check overlap
            if node.position.x < max_x
                && node.position.x + node.size.x > min_x
                && node.position.y < max_y
                && node.position.y + node.size.y > min_y
            {
                node.flags.insert(NodeFlags::SELECTED);
            }
        }
        _events.push(LogicEvent::RepaintNeeded);

        // 3. Reset
        return Some(InteractionMode::Idle);
    }
    None
}
