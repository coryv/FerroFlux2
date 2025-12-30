use glam::Vec2;

use crate::config::CanvasConfig;
use crate::interaction::InteractionMode;
use crate::math;
use crate::model::{self, GraphState, NodeFlags};
use crate::render::{DrawCommand, RenderList};
use crate::view::View;

/// High-level renderer for the FlowCanvas graph.
///
/// The `Painter` is responsible for converting the abstract graph state (Nodes, Ports, Connections)
/// into concrete drawing commands (`RenderList`) that the host application can render.
/// It handles:
/// - Grid rendering
/// - Node shape and style (including selection highlights)
/// - Port positioning and rendering
/// - Wire rendering (Bezier curves)
/// - Z-ordering (painters algorithm)
pub struct Painter;

impl Painter {
    /// Generates a list of draw commands to render the entire graph.
    ///
    /// # Arguments
    /// * `view` - The current viewport transform (pan/zoom).
    /// * `_config` - Canvas configuration (unused for now).
    /// * `graph` - The graph state to render.
    /// * `interaction_mode` - Current interaction state (used for rendering active wires/selection boxes).
    /// * `screen_size` - dimensions of the viewport in pixels (used for culling/grid).
    pub fn draw_graph<T: model::NodeData>(
        view: &View,
        config: &CanvasConfig,
        graph: &mut GraphState<T>,
        interaction_mode: &InteractionMode,
        screen_size: Vec2,
    ) -> RenderList {
        let mut draw_list = Vec::new();
        let style = &config.style;

        // 1. Background grid
        Self::draw_grid(view, style, screen_size, &mut draw_list);

        // 2. Render Connections (Behind nodes)
        for (_id, connection) in &graph.connections {
            let start_pos = graph.find_port_position(connection.from);
            let end_pos = graph.find_port_position(connection.to);

            if let (Some(start_world), Some(end_world)) = (start_pos, end_pos) {
                let screen_start = view.world_to_screen(start_world);
                let screen_end = view.world_to_screen(end_world);

                let (cp1, cp2) = math::calculate_bezier_points(screen_start, screen_end);

                // Use overrides if present, otherwise default
                let (color, width) = if let Some(override_style) = &connection.visual_style {
                    (override_style.color, override_style.width)
                } else {
                    (style.edge_default.color, style.edge_default.width)
                };

                draw_list.push(DrawCommand::Bezier {
                    start: screen_start,
                    end: screen_end,
                    cp1,
                    cp2,
                    color,
                    width,
                });
            }
        }

        // 3. Render Active Link (Dragging)
        if let InteractionMode::Linking {
            source,
            curr_pos_world,
        } = interaction_mode
        {
            // Calculate start pos
            if let Some(start_pos) = graph.find_port_position(*source) {
                let screen_start = view.world_to_screen(start_pos);
                let screen_end = view.world_to_screen(*curr_pos_world);

                let (cp1, cp2) = math::calculate_bezier_points(screen_start, screen_end);

                draw_list.push(DrawCommand::Bezier {
                    start: screen_start,
                    end: screen_end,
                    cp1,
                    cp2,
                    color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0), // Active link is white
                    width: 2.0,
                });
            }
        }

        // 4. Draw nodes based on Z-Order
        // Lazy populate draw order if empty
        if graph.draw_order.is_empty() && !graph.nodes.is_empty() {
            for (id, _) in &graph.nodes {
                graph.draw_order.push(id);
            }
        }

        for &node_id in &graph.draw_order {
            if let Some(node) = graph.nodes.get(node_id) {
                // Project world pos to screen pos
                let screen_pos = view.world_to_screen(node.position);
                let scaled_size = node.size * view.transform.zoom;

                // Resolve style: Override > Default
                let node_style = node.style.as_ref().unwrap_or(&style.node_default);

                let color = if node.flags.contains(NodeFlags::SELECTED) {
                    node_style.color * 1.2 // Highlight
                } else {
                    node_style.color
                };

                let stroke_color = if node.flags.contains(NodeFlags::SELECTED) {
                    Some(node_style.border_color * 1.5) // Highlight border
                } else {
                    Some(node_style.border_color)
                };

                let stroke_width = if node.flags.contains(NodeFlags::SELECTED) {
                    2.0
                } else {
                    1.0
                };

                draw_list.push(DrawCommand::Rect {
                    pos: screen_pos,
                    size: scaled_size,
                    color,
                    corner_radius: 5.0 * view.transform.zoom,
                    stroke_width,
                    stroke_color,
                });

                // Render Ports
                // Inputs
                let spacing_in = node.size.y / (node.inputs.len() as f32 + 1.0);
                for (i, _) in node.inputs.iter().enumerate() {
                    let local_y = spacing_in * (i as f32 + 1.0);
                    let world_pos = node.position + Vec2::new(0.0, local_y);
                    let screen_port_pos = view.world_to_screen(world_pos);
                    let port_size = Vec2::new(10.0, 10.0) * view.transform.zoom; // 10px ports

                    draw_list.push(DrawCommand::Rect {
                        pos: screen_port_pos - (port_size * 0.5), // Center it
                        size: port_size,
                        color: style.port_color,
                        corner_radius: 5.0 * view.transform.zoom, // Circle
                        stroke_width: 1.0,
                        stroke_color: Some(glam::Vec4::new(0.0, 0.0, 0.0, 1.0)),
                    });
                }

                // Outputs
                let spacing_out = node.size.y / (node.outputs.len() as f32 + 1.0);
                for (i, _) in node.outputs.iter().enumerate() {
                    let local_y = spacing_out * (i as f32 + 1.0);
                    let world_pos = node.position + Vec2::new(node.size.x, local_y);
                    let screen_port_pos = view.world_to_screen(world_pos);
                    let port_size = Vec2::new(10.0, 10.0) * view.transform.zoom; // 10px ports

                    draw_list.push(DrawCommand::Rect {
                        pos: screen_port_pos - (port_size * 0.5), // Center it
                        size: port_size,
                        color: style.port_color,
                        corner_radius: 5.0 * view.transform.zoom, // Circle
                        stroke_width: 1.0,
                        stroke_color: Some(glam::Vec4::new(0.0, 0.0, 0.0, 1.0)),
                    });
                }
            }
        }

        if let InteractionMode::BoxSelecting {
            start_pos_world,
            current_pos_world,
        } = interaction_mode
        {
            let screen_start = view.world_to_screen(*start_pos_world);
            let screen_end = view.world_to_screen(*current_pos_world);

            let min_x = screen_start.x.min(screen_end.x);
            let max_x = screen_start.x.max(screen_end.x);
            let min_y = screen_start.y.min(screen_end.y);
            let max_y = screen_start.y.max(screen_end.y);

            let pos = Vec2::new(min_x, min_y);
            let size = Vec2::new(max_x - min_x, max_y - min_y);

            // Fill
            draw_list.push(DrawCommand::Rect {
                pos,
                size,
                color: style.selection_box_color,
                corner_radius: 0.0,
                stroke_width: 1.0,
                stroke_color: Some(style.selection_box_border_color),
            });
        }

        draw_list
    }

    /// Renders an infinite background grid.
    ///
    /// This helper calculates the visible world bounds based on the viewport and
    /// renders vertical and horizontal lines.
    fn draw_grid(
        view: &View,
        style: &crate::config::CanvasStyle,
        screen_size: Vec2,
        draw_list: &mut RenderList,
    ) {
        let grid_size = 100.0; // World units

        // We need visible world bounds.
        // Screen (0,0) -> World TopLeft
        // Screen (W,H) -> World BottomRight

        let top_left_world = view.screen_to_world(Vec2::ZERO);
        let bottom_right_world = view.screen_to_world(screen_size);

        // Ensure proper min/max if zoom is negative (unlikely but safe)
        let min_x = top_left_world.x.min(bottom_right_world.x);
        let max_x = top_left_world.x.max(bottom_right_world.x);
        let min_y = top_left_world.y.min(bottom_right_world.y);
        let max_y = top_left_world.y.max(bottom_right_world.y);

        // Snap start to grid multiple
        let start_x = (min_x / grid_size).floor() * grid_size;
        let start_y = (min_y / grid_size).floor() * grid_size;

        // Vertical Lines
        let mut x = start_x;
        while x <= max_x {
            let start = view.world_to_screen(Vec2::new(x, min_y));
            let end = view.world_to_screen(Vec2::new(x, max_y));
            draw_list.push(DrawCommand::Line {
                start,
                end,
                color: style.grid_color,
                width: 1.0,
            });
            x += grid_size;
        }

        // Horizontal Lines
        let mut y = start_y;
        while y <= max_y {
            let start = view.world_to_screen(Vec2::new(min_x, y));
            let end = view.world_to_screen(Vec2::new(max_x, y));
            draw_list.push(DrawCommand::Line {
                start,
                end,
                color: style.grid_color,
                width: 1.0,
            });
            y += grid_size;
        }
    }
}
