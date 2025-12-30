use crate::components::{CronConfig, Frequency, Outbox, WorkDone};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Runtime component to track the next execution time.
#[derive(Component)]
pub struct NextRun(pub DateTime<Utc>);

#[tracing::instrument(skip(commands, query, store, work_done))]
pub fn scheduler_worker(
    mut commands: Commands,
    mut query: Query<(Entity, &CronConfig, Option<&mut NextRun>, &mut Outbox)>,
    store: Res<BlobStore>,
    mut work_done: ResMut<WorkDone>,
) {
    let now = Utc::now();

    for (entity, config, mut next_run_opt, mut outbox) in query.iter_mut() {
        match next_run_opt {
            Some(ref mut next_run) => {
                if now >= next_run.0 {
                    tracing::info!(entity = ?entity, "Triggering Cron Node");

                    // Trigger: Push generic ticket
                    let mut metadata = HashMap::new();
                    metadata.insert("trigger".to_string(), "cron".to_string());

                    if let Ok(ticket) = store.check_in_with_metadata(b"CRON_TRIGGER", metadata) {
                        outbox.queue.push_back(ticket);
                        work_done.0 = true;
                    }

                    // Calculate next run
                    let next = match config.frequency {
                        Frequency::Once => None,
                        Frequency::Minutes => Some(next_run.0 + Duration::minutes(1)),
                        Frequency::Hourly => Some(next_run.0 + Duration::hours(1)),
                        Frequency::Daily => Some(next_run.0 + Duration::days(1)),
                        Frequency::Weekly => Some(next_run.0 + Duration::days(7)),
                    };

                    match next {
                        Some(n) => {
                            next_run.0 = n;
                            tracing::debug!(entity = ?entity, next_run = %n, "Next run scheduled");
                        }
                        None => {
                            commands.entity(entity).remove::<NextRun>();
                            tracing::info!(entity = ?entity, "One-time run complete");
                        }
                    }
                }
            }
            None => {
                // Initialize first run
                // If now is already past start_at, should we run immediately?
                // For a scheduler, usually yes, or we set it to the next occurrence.
                // Let's set it to start_at and let the next iteration trigger it.
                tracing::info!(entity = ?entity, start_at = %config.start_at, "Initializing scheduler for node");
                commands.entity(entity).insert(NextRun(config.start_at));
            }
        }
    }
}
