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

#[tracing::instrument(skip(timer, store))]
pub fn janitor_worker(mut timer: ResMut<JanitorTimer>, store: Res<BlobStore>) {
    let now = Instant::now();
    // Run every 10 seconds
    if now.duration_since(timer.0) > Duration::from_secs(10) {
        // Clone the store (cheap Arc clone) to method call
        let store_inner = store.clone();
        let removed = store_inner.run_garbage_collection();
        if removed > 0 {
            tracing::info!(count = removed, "Garbage collection complete");
        }
        timer.0 = now;
    }
}
