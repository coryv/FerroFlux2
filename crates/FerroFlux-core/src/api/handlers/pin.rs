use crate::components::{NodeConfig, PinnedOutput};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use ferroflux_iam::TenantId;
use std::collections::HashMap;
use uuid::Uuid;

pub fn handle_pin_node(
    world: &mut World,
    _tenant: TenantId,
    node_id: Uuid,
    ticket_id_str: String,
) -> anyhow::Result<()> {
    tracing::info!(node_id = %node_id, ticket_id = %ticket_id_str, "Processing PinNode command");

    let ticket_uuid = Uuid::parse_str(&ticket_id_str)?;

    let mut target_entity = None;
    {
        let mut query = world.query::<(Entity, &NodeConfig)>();
        for (e, conf) in query.iter(world) {
            if conf.id == node_id {
                target_entity = Some(e);
                break;
            }
        }
    }

    if let Some(entity) = target_entity {
        if let Some(store) = world.get_resource::<BlobStore>() {
            match store.recover_ticket(&ticket_uuid) {
                Some(mut ticket) => {
                    ticket
                        .metadata
                        .insert("pinned".to_string(), "true".to_string());
                    let mut meta_update = HashMap::new();
                    meta_update.insert("pinned".to_string(), "true".to_string());
                    let _ = store.update_metadata(&ticket.id, meta_update);

                    world.entity_mut(entity).insert(PinnedOutput(ticket));
                    tracing::info!(entity = ?entity, "Node pinned successfully");
                    Ok(())
                }
                None => Err(anyhow::anyhow!("Ticket not found in store for pinning")),
            }
        } else {
            Err(anyhow::anyhow!("BlobStore not found"))
        }
    } else {
        Err(anyhow::anyhow!("Node not found for pinning"))
    }
}
