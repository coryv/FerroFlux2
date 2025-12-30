//! # Input Protocol
//!
//! This module defines the input state that the host application must pass to the Canvas every frame.
//! It includes mouse position, buttons, scroll delta, and keyboard modifiers.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// State of keyboard modifiers (Shift, Ctrl, Alt, Meta).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ModifiersState {
    /// Shift key is pressed.
    pub shift: bool,
    /// Ctrl key is pressed.
    pub ctrl: bool,
    /// Alt / Option key is pressed.
    pub alt: bool,
    /// Meta / Command / Windows key is pressed.
    pub meta: bool,
}

/// State of mouse buttons.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct MouseButtons {
    /// Left mouse button is pressed.
    pub left: bool,
    /// Right mouse button is pressed.
    pub right: bool,
    /// Middle mouse button is pressed.
    pub middle: bool,
}

/// Standard keyboard keys that the Canvas cares about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    Delete,
    Backspace,
    A,
    // Add more as needed
}

/// The input state for a single frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputState {
    /// Current position of the mouse cursor in Screen Space (pixels).
    pub mouse_pos: Vec2,
    /// State of mouse buttons.
    pub mouse_buttons: MouseButtons,
    /// Vertical scroll delta this frame (positive = up).
    pub scroll_delta: f32,
    /// State of keyboard modifiers.
    pub modifiers: ModifiersState,
    /// Keys pressed *this frame*.
    ///
    /// This should contain any keys that are currently held down or were pressed this frame.
    /// The Canvas uses this for keyboard shortcuts (e.g., Delete, Ctrl+A).
    pub pressed_keys: Vec<Key>,
    /// Size of the canvas viewport in Screen Space (pixels).
    pub screen_size: Vec2,
    /// If true, the canvas will ignore Click/Drag events (but still track mouse pos).
    /// This is used when the mouse interaction was consumed by UI content inside a node.
    pub event_consumed_by_content: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_pos: Vec2::ZERO,
            mouse_buttons: MouseButtons::default(),
            scroll_delta: 0.0,
            modifiers: ModifiersState::default(),
            pressed_keys: Vec::new(),
            screen_size: Vec2::new(800.0, 600.0), // Sound default
            event_consumed_by_content: false,
        }
    }
}
