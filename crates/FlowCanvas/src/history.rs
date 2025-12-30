use crate::model::{GraphState, NodeData};

/// Manages the Undo/Redo history of the GraphState.
///
/// This implementation uses a simple Full State Snapshot approach.
/// While less memory efficient than Command Pattern, it is robust against
/// complex state drift and guarantees correct restoration of all IDs.
pub struct HistoryManager<T> {
    undo_stack: Vec<GraphState<T>>,
    redo_stack: Vec<GraphState<T>>,
    pub max_history: usize,
}

impl<T> Default for HistoryManager<T> {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

impl<T: NodeData + Clone> HistoryManager<T> {
    /// Creates a new HistoryManager with a specified limit.
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Helper to check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Helper to check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Saving a snapshot BEFORE a destructive action.
    ///
    /// Call this *before* you mutate the graph.
    pub fn commit(&mut self, state: &GraphState<T>) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0); // Drop oldest
        }
        self.undo_stack.push(state.clone());
        self.redo_stack.clear(); // New timeline branch
    }

    /// Performs Undo.
    ///
    /// Returns true if successful (state updated), false if nothing to undo.
    pub fn undo(&mut self, state: &mut GraphState<T>) -> bool {
        if let Some(prev_state) = self.undo_stack.pop() {
            // Push CURRENT state to redo stack before overwriting
            self.redo_stack.push(state.clone());
            // Overwrite current state
            *state = prev_state;
            true
        } else {
            false
        }
    }

    /// Performs Redo.
    ///
    /// Returns true if successful (state updated), false if nothing to redo.
    pub fn redo(&mut self, state: &mut GraphState<T>) -> bool {
        if let Some(next_state) = self.redo_stack.pop() {
            // Push CURRENT state to undo stack before overwriting
            self.undo_stack.push(state.clone());
            // Overwrite current state
            *state = next_state;
            true
        } else {
            false
        }
    }
}
