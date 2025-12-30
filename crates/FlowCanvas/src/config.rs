//! # Configuration
//!
//! This module defines the configuration struct for the Canvas.

use serde::{Deserialize, Serialize};

/// Configuration parameters for the Canvas.
///
/// These settings allow the host application to tune the feel of the canvas interactions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasConfig {
    /// Multiplier for panning speed. Default: 1.0.
    pub pan_speed: f32,
    /// Multiplier for zoom speed. Default: 0.1 per scroll click.
    pub zoom_speed: f32,
    /// Distance in pixels for snapping ports together. Default: 10.0.
    pub snap_threshold: f32,
    /// Max time in ms to register a double-click. Default: 300ms.
    pub double_click_time_ms: u64,
    /// Visual styling configuration.
    #[serde(default)]
    pub style: CanvasStyle,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            pan_speed: 1.0,
            zoom_speed: 0.1,
            snap_threshold: 10.0,
            double_click_time_ms: 300,
            style: CanvasStyle::default(),
        }
    }
}

/// Visual styling configuration for the Canvas.
///
/// This struct defines the colors used for rendering the graph.
/// It uses `glam::Vec4` for RGBA colors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanvasStyle {
    /// Background color of the canvas.
    pub background_color: glam::Vec4,
    /// Color of the grid lines.
    pub grid_color: glam::Vec4,
    /// Default style for nodes.
    #[serde(default)]
    pub node_default: NodeStyle,
    /// Default style for edges (wires).
    #[serde(default)]
    pub edge_default: EdgeStyle,
    /// Color of the ports.
    pub port_color: glam::Vec4,
    /// Color of the selection box (fill).
    pub selection_box_color: glam::Vec4,
    /// Color of the selection box (border).
    pub selection_box_border_color: glam::Vec4,
}

impl Default for CanvasStyle {
    fn default() -> Self {
        Self {
            background_color: glam::Vec4::new(0.1, 0.1, 0.1, 1.0),
            grid_color: glam::Vec4::new(0.2, 0.2, 0.2, 1.0),
            node_default: NodeStyle::default(),
            edge_default: EdgeStyle::default(),
            port_color: glam::Vec4::new(0.7, 0.7, 0.7, 1.0),
            selection_box_color: glam::Vec4::new(0.3, 0.3, 0.6, 0.2),
            selection_box_border_color: glam::Vec4::new(0.4, 0.4, 0.8, 0.5),
        }
    }
}

/// Visual style for a Node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeStyle {
    /// Fill color of the node.
    pub color: glam::Vec4,
    /// Border color of the node.
    pub border_color: glam::Vec4,
    /// Color of the text label.
    pub text_color: glam::Vec4,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            color: glam::Vec4::new(0.15, 0.15, 0.15, 1.0),
            border_color: glam::Vec4::new(0.5, 0.5, 0.5, 1.0),
            text_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

/// Visual style for an Edge (Wire).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeStyle {
    /// Color of the wire.
    pub color: glam::Vec4,
    /// Width of the wire in screen pixels.
    pub width: f32,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            color: glam::Vec4::new(0.8, 0.8, 0.8, 1.0),
            width: 2.0,
        }
    }
}
