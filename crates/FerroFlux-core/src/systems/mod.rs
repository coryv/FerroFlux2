use bevy_ecs::prelude::*;

pub mod agent;
pub mod api_worker;
pub mod compute;
pub mod connectors;
pub mod control;
pub mod execution;
pub mod gateway;
pub mod io;
pub mod janitor;
pub mod logic;
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
pub use logic::*;
pub use observability::*;
pub use scheduler::*;
pub use transport::*;

/// Registers all core systems to the schedule.
pub fn register_core_systems(schedule: &mut Schedule) {
    schedule.add_systems((
        scheduler::scheduler_worker,
        gateway::ingest_webhooks,
        logic::switch_worker_safe,
        logic::script_worker,
        agent::agent_prep,
        agent::agent_exec,
        agent::agent_post,
        io::http_worker,
    ));

    schedule.add_systems((
        transport::update_graph_topology, // Optimization: Needs to run before transport
        transport::transport_worker,
        janitor::janitor_worker,
        manipulation::splitter_worker,
        compute::wasm_worker,
        observability::telemetry_worker,
    ));

    schedule.add_systems((
        manipulation::aggregator_worker,
        manipulation::transform_worker,
        manipulation::stats_worker,
        manipulation::window_worker,
        manipulation::expression_worker,
        control::checkpoint_worker,
        connectors::rss_worker,
        connectors::xml_worker,
        connectors::ftp_worker,
        connectors::ssh_worker,
    ));
}
