use bevy_ecs::prelude::Resource;
use std::collections::HashMap;
use uuid::Uuid;

pub struct SseConnectionHandle {
    pub abort_handle: tokio::task::AbortHandle,
    pub config_hash: u64,
}

#[derive(Resource, Default)]
pub struct SseTriggerRegistry {
    /// Maps (workflow_id, node_id) -> Connection handle
    pub connections: HashMap<(String, Uuid), SseConnectionHandle>,
}

impl SseTriggerRegistry {
    pub fn abort_all(&mut self) {
        for (_, handle) in self.connections.drain() {
            handle.abort_handle.abort();
        }
    }
}

impl Drop for SseTriggerRegistry {
    fn drop(&mut self) {
        self.abort_all();
    }
}
