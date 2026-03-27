use crate::components::pipeline::PipelineNode;
use anyhow::Result;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub fn handle_simulate_node(
    world: &mut World,
    tenant: ferroflux_iam::TenantId,
    node_id: uuid::Uuid,
    definition_id: String,
    config: HashMap<String, serde_json::Value>,
    input_payload: serde_json::Value,
    trace_id: String,
    mock_config: HashMap<String, crate::components::shadow::MockConfig>,
) -> Result<()> {
    // Wrap inner logic to capture errors
    let result = (|| -> Result<()> {
        // Use the definition and config provided directly in the command (Draft Mode support)

        // Store Input Payload
        let ticket = if let Some(store) = world.get_resource::<crate::store::BlobStore>().cloned() {
            let payload_bytes =
                serde_json::to_vec(&input_payload).unwrap_or_else(|_| b"{}".to_vec());
            let mut metadata = HashMap::new();
            metadata.insert("trace_id".to_string(), trace_id.clone());
            metadata.insert("shadow".to_string(), "true".to_string());

            store.check_in_with_metadata(&payload_bytes, metadata)?
        } else {
            return Err(anyhow::anyhow!("BlobStore resource not found"));
        };

        // Spawn Ephemeral Mutation
        let shadow_node = PipelineNode {
            definition_id: definition_id.clone(),
            config: config.clone(),
            execution_context: Default::default(),
        };

        // Prepare Input
        let mut inbox = crate::components::Inbox::default();
        inbox.queue.push_back(ticket);

        // Spawn
        world.spawn((
            shadow_node,
            inbox,
            crate::components::Outbox::default(),
            crate::components::shadow::ShadowExecution {
                mocked_tools: mock_config,
            },
            crate::components::NodeConfig {
                id: node_id,
                name: "Shadow Node".to_string(),
                node_type: "Shadow".to_string(),
                workflow_id: "shadow".to_string(),
                tenant_id: tenant.clone(),
            },
            // We might want a cleanup component or TTL so these don't pile up?
            // For now, rely on "Janitor"? Janitor cleans traces, not entities.
            // We should add `Ephemeral` component.
        ));

        tracing::info!(%node_id, "Spawned ephemeral shadow node");
        Ok(())
    })();

    if let Err(ref e) = result {
        // Emit error event so client doesn't time out
        if let Some(bus) = world.get_resource::<crate::api::events::SystemEventBus>() {
            let _ = bus.0.send(crate::api::events::SystemEvent::NodeError {
                trace_id: trace_id.clone(),
                node_id,
                error: e.to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }
    }

    result
}
