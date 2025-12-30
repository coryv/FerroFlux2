use crate::components::{Outbox, WorkDone};
use crate::store::SecureTicket;
use async_channel::{Receiver, Sender};
use bevy_ecs::prelude::*;
use once_cell::sync::OnceCell;
use uuid::Uuid;

// Global Queue for Webhooks -> ECS
#[allow(clippy::type_complexity)]
pub static WEBHOOK_QUEUE: OnceCell<(Sender<(Uuid, SecureTicket)>, Receiver<(Uuid, SecureTicket)>)> =
    OnceCell::new();

// Note: run_webhook_server has been moved to the App crate to keep Core headless.

#[tracing::instrument(skip(outbox_query, node_router, work_done))]
pub fn ingest_webhooks(
    mut outbox_query: Query<&mut Outbox>,
    node_router: Res<crate::resources::NodeRouter>,
    mut work_done: ResMut<WorkDone>,
) {
    let queue = match WEBHOOK_QUEUE.get() {
        Some((_, rx)) => rx,
        None => return,
    };
    while let Ok((node_id, ticket)) = queue.try_recv() {
        // O(1) Lookup
        if let Some(&entity) = node_router.0.get(&node_id) {
            if let Ok(mut outbox) = outbox_query.get_mut(entity) {
                tracing::info!(webhook_id = %node_id, entity = ?entity, "Routing Webhook to Node");
                outbox.queue.push_back(ticket.clone());
                work_done.0 = true;
            } else {
                tracing::warn!(entity = ?entity, "Found Node in Router, but missing Outbox component");
            }
        } else {
            tracing::debug!(webhook_id = %node_id, "No node found for webhook");
        }
    }
}
