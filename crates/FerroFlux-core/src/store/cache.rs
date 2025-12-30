use bevy_ecs::prelude::*;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Resource, Clone, Default)]
pub struct IntegrationCache {
    // Key: tenant_id:connection_slug:action_name
    // Value: (JSON string result, Timestamp)
    pub cache: Arc<DashMap<String, (String, Instant)>>,
}
