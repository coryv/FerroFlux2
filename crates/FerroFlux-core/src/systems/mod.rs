use bevy_ecs::prelude::*;

pub mod agent;
pub mod api_worker;
pub mod control;
pub mod execution;
pub mod gateway;
pub mod io;
pub mod janitor;
pub mod manipulation;
pub mod observability;
pub mod pipeline;
pub mod scheduler;
pub mod transport;
pub mod utils;

pub use agent::*;
pub use gateway::*;
pub use io::*;
pub use janitor::*;
pub use observability::*;
pub use scheduler::*;
pub use transport::*;

/// Registers all core systems to the schedule.
pub fn register_core_systems(schedule: &mut Schedule) {
    // 1. Ingest
    schedule.add_systems((
        scheduler::scheduler_worker,
        gateway::ingest_triggers,
    ));

    // 2. Topology Rebuild (Must run before transport)
    schedule.add_systems(
        transport::update_graph_topology
            .after(gateway::ingest_triggers)
    );

    // 3. Transport (Move data from Outbox to Inbox)
    schedule.add_systems(
        transport::transport_worker
            .after(transport::update_graph_topology)
    );

    // 4. Execution (Process Inbox)
    schedule.add_systems((
        agent::agent_prep,
        agent::agent_exec,
        agent::agent_post,
        pipeline::pipeline_execution_system,
    ).after(transport::transport_worker));

    // 5. Cleanup/Telemetry
    schedule.add_systems((
        janitor::janitor_worker,
        observability::telemetry_worker,
        manipulation::window_worker,
        control::checkpoint_worker,
    ).after(pipeline::pipeline_execution_system));
}
