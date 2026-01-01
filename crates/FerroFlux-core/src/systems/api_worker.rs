use crate::api::{ApiCommand, ApiReceiver};
use crate::components::WorkDone;
use crate::components::{Inbox, NodeConfig};
use crate::graph_loader::load_graph_from_str;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;

/// System: API Command Consumer
///
/// **Role**: Bridges the Async World (Axum Web Server) and the Sync World (Bevy ECS).
///
/// **Mechanism**:
/// - The Web Server sends `ApiCommand` enums into a generic `async_channel`.
/// - This system drains that channel every tick.
/// - It performs "Exclusive World Access" operations like Spawning Entities (`LoadGraph`),
///   which cannot be done easily from async handlers.
#[tracing::instrument(skip(world))]
pub fn api_command_worker(world: &mut World) {
    // 1. Check if resource exists
    if !world.contains_resource::<ApiReceiver>() {
        return;
    }

    // 2. Drain channel directly
    // We can't move the receiver out of the resource easily without removing it.
    // So we'll get a reference and try_recv looping.

    // NOTE: world.resource::<ApiReceiver>() borrows world immutably.
    // But we need world mutably later to load_graph.
    // Solution: Clone the receiver. It's an async_channel::Receiver which is cheap to clone (Arc-like).

    let receiver = world.resource::<ApiReceiver>().0.clone();

    while let Ok(cmd) = receiver.try_recv() {
        match cmd {
            ApiCommand::LoadGraph(tenant, yaml) => {
                tracing::info!("Processing LoadGraph command");
                if let Err(e) = load_graph_from_str(world, tenant, &yaml) {
                    tracing::error!(error = %e, "Error loading graph");
                } else {
                    tracing::info!("Graph loaded successfully");
                }
            }
            ApiCommand::TriggerNode(_tenant, uuid, payload) => {
                tracing::info!(node_id = %uuid, "Processing TriggerNode command");

                // Find entity by UUID
                // Querying world
                let mut target_entity = None;

                // Scope for query to drop borrow
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
                    // Get Store
                    if let Some(store) = world.get_resource::<BlobStore>().cloned() {
                        // Serialize payload to bytes
                        let payload_bytes =
                            serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());

                        if let Ok(ticket) = store.check_in(&payload_bytes) {
                            // Check Node Type to decide Inbox vs Outbox
                            let is_source = if let Some(conf) = world.get::<NodeConfig>(e) {
                                conf.node_type == "Webhook" || conf.node_type == "Cron"
                            } else {
                                false
                            };

                            if is_source {
                                if let Some(mut outbox) =
                                    world.get_mut::<crate::components::Outbox>(e)
                                {
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
                } else {
                    tracing::warn!(node_id = %uuid, "Node not found for trigger");
                }
            }
            ApiCommand::TriggerWorkflow(_tenant, workflow_id, payload) => {
                tracing::info!(workflow_id = %workflow_id, "Processing TriggerWorkflow command");

                let mut target_entity = None;

                {
                    // Find a candidate Start Node (Webhook) for this workflow
                    let mut query = world.query::<(Entity, &NodeConfig)>();
                    for (e, conf) in query.iter(world) {
                        // Check if it belongs to workflow AND is a Webhook (for now, or any trigger)
                        // Note: We prioritize Webhook, but could default to a "Manual Trigger" node if we had one.
                        // For now: Just look for matching workflow_id.
                        if let Some(wf_id) = &conf.workflow_id
                            && wf_id == &workflow_id
                        {
                            // Match! Prefer Webhook if possible, but take valid one.
                            // If Node is Webhook type?
                            if conf.node_type == "Webhook" {
                                target_entity = Some(e);
                                break;
                            }
                            // Fallback: If we haven't found a webhook yet, take this one temporarily?
                            // Better to look for specific trigger types.
                        }
                    }
                }

                if let Some(e) = target_entity {
                    // Get Store
                    if let Some(store) = world.get_resource::<BlobStore>().cloned() {
                        let payload_bytes =
                            serde_json::to_vec(&payload).unwrap_or_else(|_| b"{}".to_vec());
                        if let Ok(ticket) = store.check_in(&payload_bytes) {
                            // Check Node Type
                            let is_source = if let Some(conf) = world.get::<NodeConfig>(e) {
                                conf.node_type == "Webhook" || conf.node_type == "Cron"
                            } else {
                                false
                            };

                            if is_source {
                                if let Some(mut outbox) =
                                    world.get_mut::<crate::components::Outbox>(e)
                                {
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
                } else {
                    tracing::warn!(workflow_id = %workflow_id, "No suitable start node found for workflow");
                }
            }
            ApiCommand::PinNode(_tenant, node_id, ticket_id_str) => {
                tracing::info!(node_id = %node_id, ticket_id = %ticket_id_str, "Processing PinNode command");

                // 1. Resolve Ticket UUID
                // The ticket_id_str passed is just the UUID part of the ticket usually, or maybe we need to lookup?
                // The prompt says "ticket_id".
                if let Ok(ticket_uuid) = uuid::Uuid::parse_str(&ticket_id_str) {
                    // 2. Find Entity
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
                        // 3. Verify Ticket Exists & Construct PinnedOutput
                        // We need to re-construct the SecureTicket.
                        // PROBLEM: Store only stores (Data, Timestamp). It doesn't store the metadata/hash index directly to look up *by ID* and get Hash.
                        // BlobStore uses DashMap<Uuid, StoreEntry>.
                        // BUT `SecureTicket` requires `integrity_hash`.
                        // We can't generate `SecureTicket` from just UUID without creating a new hash (requires reading data).

                        if let Some(store) = world.get_resource::<BlobStore>() {
                            // We need to access store to read data and re-hash.
                            // `store.claim` needs a ticket. Circular.
                            // We need a way to "Upgrade" a UUID to a Ticket if it exists in store.
                            // Add helper to BlobStore? Or just read manually here? `BlobStore` fields are not pub (store is private).
                            // I must add a new method to `BlobStore` to `get_ticket(uuid)`.

                            // For now, I will modify BlobStore in next step.
                            // I'll emit a "TODO" or call a method I WILL create. `recover_ticket`.

                            match store.recover_ticket(&ticket_uuid) {
                                Some(mut ticket) => {
                                    // 4. Mark as Pinned in Metadata
                                    ticket
                                        .metadata
                                        .insert("pinned".to_string(), "true".to_string());

                                    // Persist to StoreEntry so Janitor sees it
                                    let mut meta_update = std::collections::HashMap::new();
                                    meta_update.insert("pinned".to_string(), "true".to_string());

                                    let _ = store.update_metadata(&ticket.id, meta_update);

                                    world
                                        .entity_mut(entity)
                                        .insert(crate::components::PinnedOutput(ticket));
                                    tracing::info!(entity = ?entity, "Node pinned successfully");
                                }
                                None => {
                                    tracing::error!(ticket_id = %ticket_uuid, "Ticket not found in store for pinning");
                                }
                            }
                        }
                    } else {
                        tracing::warn!(node_id = %node_id, "Node not found for pinning");
                    }
                }
            }
            ApiCommand::ReloadDefinitions => {
                tracing::info!("Processing ReloadDefinitions command");

                let path_opt = world.get_resource::<crate::api::PlatformPath>().cloned();

                if let Some(path_res) = path_opt {
                    let path = path_res.0;
                    tracing::info!(path = ?path, "Reloading definitions from path");

                    // 1. Refresh DefinitionRegistry
                    let def_registry =
                        world.get_resource_mut::<crate::resources::registry::DefinitionRegistry>();
                    if let Some(mut registry) = def_registry {
                        registry.clear();
                        if let Err(e) = registry.load_from_dir(&path) {
                            tracing::error!(error = %e, "Failed to reload definitions");
                        }
                    }

                    // 2. Re-bridge to NodeRegistry
                    let def_registry_clone = world
                        .get_resource::<crate::resources::registry::DefinitionRegistry>()
                        .cloned();

                    if let Some(defs) = def_registry_clone {
                        let node_registry =
                            world.get_resource_mut::<crate::resources::registry::NodeRegistry>();
                        if let Some(mut registry) = node_registry {
                            registry.clear();

                            // Re-register core nodes
                            crate::nodes::register_core_nodes(&mut registry);

                            // Re-register YAML nodes
                            for (id, def) in &defs.definitions {
                                registry.register(
                                    id,
                                    Box::new(crate::nodes::yaml_factory::YamlNodeFactory::new(
                                        def.clone(),
                                    )),
                                );
                            }
                            tracing::info!(
                                count = defs.definitions.len(),
                                "Node factories reloaded"
                            );
                        }
                    }
                } else {
                    tracing::warn!("PlatformPath resource not found, cannot reload");
                }
            }
        }
    }
}
