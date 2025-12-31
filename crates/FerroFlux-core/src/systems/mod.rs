use bevy_ecs::prelude::*;

pub mod agent_exec;
pub mod agent_post;
pub mod agent_prep;
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
pub mod pipeline;
pub mod scheduler;
pub mod transport;
pub mod utils;

pub use agent_exec::*;
pub use agent_post::*;
pub use agent_prep::*;
pub use gateway::*;
pub use io::*;
pub use janitor::*;
pub use logic::*;
pub use scheduler::*;
pub use transport::*;

/// Registers all core systems to the schedule.
pub fn register_core_systems(schedule: &mut Schedule) {
    schedule.add_systems((
        scheduler::scheduler_worker,
        gateway::ingest_webhooks,
        logic::switch_worker_safe,
        logic::script_worker,
        agent_prep::agent_prep,
        agent_exec::agent_exec,
        agent_post::agent_post,
        io::http_worker,
    ));

    schedule.add_systems((
        transport::update_graph_topology, // Optimization: Needs to run before transport
        transport::transport_worker,
        janitor::janitor_worker,
        manipulation::splitter_worker,
        compute::wasm_worker,
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
