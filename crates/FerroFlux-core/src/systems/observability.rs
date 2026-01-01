use crate::api::events::SystemEventBus;
use crate::components::observability::*;
use crate::resources::WorkDone;
use bevy_ecs::prelude::*;
use chrono::Utc;
use uuid::Uuid;

/// System: Telemetry System (The Observer)
///
/// **Role**: Monitors the state of the engine and emits telemetry events.
/// It also manages the lifecycle of `Trace` entities.
#[tracing::instrument(skip(_event_bus, _query, _work_done))]
pub fn telemetry_worker(
    _event_bus: Res<SystemEventBus>,
    _query: Query<(Entity, &Trace, &TraceNode, &TraceStart, &TraceInput)>,
    mut _work_done: ResMut<WorkDone>,
) {
    // This system could theoretically watch for changes and emit events.
    // For now, let's focus on emitting NodeTelemetry when a node completes.
    // However, NodeTelemetry usually requires the result of the node execution.
    // If we use the Trace entity to store the "Current Result", we can emit it here.
}

/// Helper to create a new Trace entity.
pub fn spawn_trace(
    commands: &mut Commands,
    trace_id: Uuid,
    start_node: Uuid,
    input: serde_json::Value,
) -> Entity {
    commands
        .spawn((
            Trace(trace_id),
            TraceNode(start_node),
            TraceStart(Utc::now()),
            TraceInput(input),
        ))
        .id()
}
