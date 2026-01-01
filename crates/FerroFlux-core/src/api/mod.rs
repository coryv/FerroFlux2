pub mod events;
pub mod handlers;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiCommand {
    LoadGraph(ferroflux_iam::TenantId, String),
    TriggerNode(ferroflux_iam::TenantId, uuid::Uuid, serde_json::Value),
    TriggerWorkflow(ferroflux_iam::TenantId, String, serde_json::Value),
    PinNode(ferroflux_iam::TenantId, uuid::Uuid, String),
    ReloadDefinitions,
    SimulateNode {
        tenant_id: ferroflux_iam::TenantId,
        node_id: uuid::Uuid,
        input_ticket: uuid::Uuid,
        trace_id: String,
        mock_config: std::collections::HashMap<String, crate::components::shadow::MockConfig>,
    },
}

#[derive(bevy_ecs::prelude::Resource)]
pub struct ApiReceiver(pub async_channel::Receiver<ApiCommand>);

#[derive(bevy_ecs::prelude::Resource, Clone, Debug)]
pub struct PlatformPath(pub std::path::PathBuf);
