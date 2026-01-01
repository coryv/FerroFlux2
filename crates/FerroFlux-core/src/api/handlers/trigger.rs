use crate::components::{Inbox, NodeConfig, Outbox, WorkDone};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use ferroflux_iam::TenantId;
use serde_json::Value;
use uuid::Uuid;

pub fn handle_trigger_node(
    world: &mut World,
    _tenant: TenantId,
    uuid: Uuid,
    payload: Value,
) -> anyhow::Result<()> {
    tracing::info!(node_id = %uuid, "Processing TriggerNode command");

    let mut target_entity = None;
    {
        let mut query = world.query::<(Entity, &NodeConfig)>();
        for (e, conf) in query.iter(world) {
            if conf.id == uuid {
                target_entity = Some(e);
                break;
            }
        }
    }

    if let Some(e) = target_entity {
        if let Some(store) = world.get_resource::<BlobStore>().cloned() {
            let payload_bytes = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
            if let Ok(ticket) = store.check_in(&payload_bytes) {
                let is_source = if let Some(conf) = world.get::<NodeConfig>(e) {
                    conf.node_type == "Webhook" || conf.node_type == "Cron"
                } else {
                    false
                };

                if is_source {
                    if let Some(mut outbox) = world.get_mut::<Outbox>(e) {
                        outbox.queue.push_back((None, ticket));
                        tracing::info!(entity = ?e, "Trigger sent to OUTBOX (Source Node)");
                        if let Some(mut wd) = world.get_resource_mut::<WorkDone>() {
                            wd.0 = true;
                        }
                    }
                } else if let Some(mut inbox) = world.get_mut::<Inbox>(e) {
                    inbox.queue.push_back(ticket);
                    tracing::info!(entity = ?e, "Trigger sent to INBOX");
                    if let Some(mut wd) = world.get_resource_mut::<WorkDone>() {
                        wd.0 = true;
                    }
                }
            }
        }
        Ok(())
    } else {
        Err(anyhow::anyhow!("Node not found for trigger"))
    }
}

pub fn handle_trigger_workflow(
    world: &mut World,
    _tenant: TenantId,
    workflow_id: String,
    payload: Value,
) -> anyhow::Result<()> {
    tracing::info!(workflow_id = %workflow_id, "Processing TriggerWorkflow command");

    let mut target_entity = None;
    {
        let mut query = world.query::<(Entity, &NodeConfig)>();
        for (e, conf) in query.iter(world) {
            if let Some(wf_id) = &conf.workflow_id {
                if wf_id == &workflow_id && conf.node_type == "Webhook" {
                    target_entity = Some(e);
                    break;
                }
            }
        }
    }

    if let Some(e) = target_entity {
        if let Some(store) = world.get_resource::<BlobStore>().cloned() {
            let payload_bytes = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
            if let Ok(ticket) = store.check_in(&payload_bytes) {
                let is_source = if let Some(conf) = world.get::<NodeConfig>(e) {
                    conf.node_type == "Webhook" || conf.node_type == "Cron"
                } else {
                    false
                };

                if is_source {
                    if let Some(mut outbox) = world.get_mut::<Outbox>(e) {
                        outbox.queue.push_back((None, ticket));
                        tracing::info!(entity = ?e, "Workflow trigger sent to OUTBOX (Source)");
                        if let Some(mut wd) = world.get_resource_mut::<WorkDone>() {
                            wd.0 = true;
                        }
                    }
                } else if let Some(mut inbox) = world.get_mut::<Inbox>(e) {
                    inbox.queue.push_back(ticket);
                    tracing::info!(entity = ?e, "Workflow trigger sent to INBOX");
                    if let Some(mut wd) = world.get_resource_mut::<WorkDone>() {
                        wd.0 = true;
                    }
                }
            }
        }
        Ok(())
    } else {
        Err(anyhow::anyhow!("No suitable start node found for workflow"))
    }
}
