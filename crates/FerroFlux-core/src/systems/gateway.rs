use crate::components::{Outbox, WorkDone};
use crate::traits::trigger::TriggerReceiver;
use bevy_ecs::prelude::*;

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
