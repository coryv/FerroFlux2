use crate::components::{Outbox, WorkDone};
use crate::store::SecureTicket;
use crate::traits::trigger::{TriggerEvent, TriggerReceiver, TriggerSender};
use async_channel::{Receiver, Sender};
use bevy_ecs::prelude::*;
use once_cell::sync::OnceCell;
use uuid::Uuid;

// Global Queue for Webhooks -> ECS
#[allow(clippy::type_complexity)]
pub static WEBHOOK_QUEUE: OnceCell<(Sender<(Uuid, SecureTicket)>, Receiver<(Uuid, SecureTicket)>)> =
    OnceCell::new();

// Note: run_webhook_server has been moved to the App crate to keep Core headless.

#[tracing::instrument(skip(outbox_query, node_router, work_done, trigger_receiver))]
pub fn ingest_triggers(
    mut outbox_query: Query<&mut Outbox>,
    node_router: Res<crate::resources::NodeRouter>,
    mut work_done: ResMut<WorkDone>,
    trigger_receiver: Res<TriggerReceiver>,
) {
    while let Ok(event) = trigger_receiver.0.try_recv() {
        let node_id = event.trigger_id;
        let ticket = event.payload;

        // O(1) Lookup
        if let Some(&entity) = node_router.0.get(&node_id) {
            if let Ok(mut outbox) = outbox_query.get_mut(entity) {
                tracing::info!(trigger_id = %node_id, entity = ?entity, "Routing Trigger to Node");
                outbox.queue.push_back((None, ticket.clone()));
                work_done.0 = true;
            } else {
                tracing::warn!(entity = ?entity, "Found Node in Router, but missing Outbox component");
            }
        } else {
            tracing::debug!(trigger_id = %node_id, "No node found for trigger");
        }
    }
}

/// Bridges the legacy static WEBHOOK_QUEUE to the new unified TriggerSender resource.
/// This ensures backward compatibility for existing API endpoints pushing to the static queue.
pub fn bridge_webhook_queue(sender: Res<TriggerSender>) {
    let queue = match WEBHOOK_QUEUE.get() {
        Some((_, rx)) => rx,
        None => return,
    };

    while let Ok((node_id, ticket)) = queue.try_recv() {
        let event = TriggerEvent {
            trigger_id: node_id,
            payload: ticket,
        };
        // We use try_send to avoid blocking the ECS tick if channel is full,
        // though strictly speaking we want high reliability here.
        if let Err(e) = sender.0.try_send(event) {
            tracing::error!(error = %e, "Failed to bridge webhook to trigger channel (channel full/closed?)");
        }
    }
}
