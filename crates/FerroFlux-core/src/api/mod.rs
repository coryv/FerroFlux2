pub mod events;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiCommand {
    LoadGraph(crate::domain::TenantId, String),
    TriggerNode(crate::domain::TenantId, uuid::Uuid, serde_json::Value),
    TriggerWorkflow(crate::domain::TenantId, String, serde_json::Value),
    PinNode(crate::domain::TenantId, uuid::Uuid, String),
}

#[derive(bevy_ecs::prelude::Resource)]
pub struct ApiReceiver(pub async_channel::Receiver<ApiCommand>);
