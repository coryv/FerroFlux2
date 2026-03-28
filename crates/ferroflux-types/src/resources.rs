use bevy_ecs::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Resource, Clone, Debug)]
pub struct TokioRuntime(pub tokio::runtime::Handle);

#[derive(Resource, Clone)]
pub struct GlobalHttpClient {
    pub client: reqwest::Client,
}

impl Default for GlobalHttpClient {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap();
        Self { client }
    }
}

pub struct SseConnectionHandle {
    pub abort_handle: tokio::task::AbortHandle,
    pub config_hash: u64,
}

#[derive(Resource, Default)]
pub struct SseTriggerRegistry {
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
