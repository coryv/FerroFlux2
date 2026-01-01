use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use std::time::{Duration, Instant};

// A local resource or component could track last run time,
// but since Systems run every tick, we can use a simpler approach:
// We'll use a local static timer check, or just a resource if we want to be pure ECS.
// For "The Janitor System", let's use a Resource to track timing.

#[derive(Resource)]
pub struct JanitorTimer(pub Instant);

impl Default for JanitorTimer {
    fn default() -> Self {
        Self(Instant::now())
    }
}

#[tracing::instrument(skip(commands, timer, store, trace_query))]
pub fn janitor_worker(
    mut commands: Commands,
    mut timer: ResMut<JanitorTimer>,
    store: Res<BlobStore>,
    trace_query: Query<(Entity, &crate::components::observability::TraceStart)>,
) {
    let now = Instant::now();
    // Run every 10 seconds
    if now.duration_since(timer.0) > Duration::from_secs(10) {
        // Clone the store (cheap Arc clone) to method call
        let store_inner = store.clone();
        let removed = store_inner.run_garbage_collection();
        if removed > 0 {
            tracing::info!(count = removed, "Garbage collection complete");
        }

        // 2. Prune old Trace entities (e.g. older than 1 hour)
        let trace_ttl = chrono::Duration::hours(1);
        let now_utc = chrono::Utc::now();
        for (entity, start) in trace_query.iter() {
            if now_utc.signed_duration_since(start.0) > trace_ttl {
                commands.entity(entity).despawn();
                tracing::debug!(entity = ?entity, "Pruned expired Trace entity");
            }
        }

        timer.0 = now;
    }
}
