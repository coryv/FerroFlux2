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
    schedule.add_systems((
        scheduler::scheduler_worker,
        gateway::ingest_triggers,
        agent::agent_prep,
        agent::agent_exec,
        agent::agent_post,
        pipeline::pipeline_execution_system,
    ));

    schedule.add_systems((
        transport::update_graph_topology, // Optimization: Needs to run before transport
        transport::transport_worker,
        janitor::janitor_worker,
        observability::telemetry_worker,
    ));

    schedule.add_systems((manipulation::window_worker, control::checkpoint_worker));
}
