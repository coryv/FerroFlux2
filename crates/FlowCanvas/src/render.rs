//! # Rendering System
//!
//! This module acts as the "Instruction Set Architecture" for the GPU.
//! Instead of drawing directly, the Canvas outputs a display list of `DrawCommand`s.
//! The host application (Egui, WGPU, etc.) is responsible for interpreting these commands and drawing pixels.

use glam::{Vec2, Vec4};
use serde::{Deserialize, Serialize};

/// A single drawing primitive.
///
/// Coordinates are in **Screen Space** (Pixels).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DrawCommand {
    /// A filled rounded rectangle with an optional stroke.
    Rect {
        /// Top-left position in screen pixels.
        pos: Vec2,
        /// Size in screen pixels.
        size: Vec2,
        /// Fill color (RGBA, 0.0 - 1.0).
        color: Vec4,
        /// Radius of the corners in pixels.
        corner_radius: f32,
        /// Width of the border stroke in pixels.
        stroke_width: f32,
        /// Color of the border stroke.
        stroke_color: Option<Vec4>,
    },
    /// A straight line segment.
    Line {
        /// Start point in screen pixels.
        start: Vec2,
        /// End point in screen pixels.
        end: Vec2,
        /// Line color (RGBA, 0.0 - 1.0).
        color: Vec4,
        /// Line thickness in pixels.
        width: f32,
    },
    /// Text to be rendered.
    Text {
        /// Top-left position in screen pixels.
        pos: Vec2,
        /// The styling and layout of text is handled by the consumer.
        text: String,
        /// Text color.
        color: Vec4,
        /// Font size in pixels (approximate).
        size: f32,
    },
    /// A cubic Bezier curve, primarily for connection wires.
    Bezier {
        /// Start point.
        start: Vec2,
        /// Control Point 1.
        cp1: Vec2,
        /// Control Point 2.
        cp2: Vec2,
        /// End point.
        end: Vec2,
        /// Curve color.
        color: Vec4,
        /// Curve thickness.
        width: f32,
    },
}

/// A list of draw commands representing the current frame.
pub type RenderList = Vec<DrawCommand>;
