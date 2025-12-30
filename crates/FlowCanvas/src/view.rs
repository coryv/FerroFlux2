//! # Viewport System
//!
//! This module handles the "infinite canvas" mathematics.
//! It provides utilities to transform between World Space (the infinite grid) and Screen Space (the pixels on the monitor).

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Represents the current camera state: where we are looking (Pan) and how close (Zoom).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform {
    /// The translation offset of the canvas.
    /// A positive value moves the canvas right/down.
    pub pan: Vec2,
    /// The scale factor.
    /// - 1.0 = 100% scale.
    /// - Greater than 1.0 = Zoomed In.
    /// - Less than 1.0 = Zoomed Out.
    pub zoom: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

/// The View struct combines the Transform with the actual Viewport size (window size).
/// It serves as the single source of truth for coordinate conversions.
pub struct View {
    /// The camera transform.
    pub transform: Transform,
    /// The size of the visible area in pixels.
    pub viewport_size: Vec2,
}

impl View {
    /// Creates a new View system.
    pub fn new(transform: Transform, viewport_size: Vec2) -> Self {
        Self {
            transform,
            viewport_size,
        }
    }

    /// Converts a point from **World Space** (infinite grid) to **Screen Space** (window pixels).
    ///
    /// Formula: `Screen = (World * Zoom) + Pan`
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos * self.transform.zoom) + self.transform.pan
    }

    /// Converts a point from **Screen Space** (window pixels) to **World Space** (infinite grid).
    ///
    /// Formula: `World = (Screen - Pan) / Zoom`
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos - self.transform.pan) / self.transform.zoom
    }
}
