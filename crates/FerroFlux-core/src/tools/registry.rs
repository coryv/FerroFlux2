use crate::tools::Tool;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Resource that holds all available Tools.
#[derive(Resource, Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Registers a new tool.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let id = tool.id().to_string();
        self.tools.insert(id.clone(), Arc::new(tool));
        tracing::info!("Registered tool: {}", id);
    }

    /// Retrieves a tool by its ID.
    pub fn get(&self, id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(id).cloned()
    }

    /// Lists all registered tool IDs.
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}
