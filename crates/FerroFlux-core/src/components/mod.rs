/// ECS Components representing the configuration and state of Workflow Nodes.
///
/// Each struct here roughly corresponds to a "Node Type" in the visual graph editor.
/// These components are attached to Entities and processed by Systems in `src/systems/`.
pub mod agent;
pub mod compute;
pub mod connectors;
pub mod control;
pub mod core;
pub mod integration;
pub mod io;
pub mod logic;
pub mod manipulation;
pub mod pipeline;
pub mod schema;
pub mod security;

// Re-export everything for backward compatibility
pub use self::agent::*;
pub use self::core::*;
pub use self::integration::*;
pub use self::io::*;
pub use self::logic::*;
pub use self::schema::*;
pub use self::security::*;

// Re-export resources that were moved, to preserve import paths if possible
// Note: This relies on `crate::resources::*` being accessible if we import it.
// However, circular deps might be issue if resources imports components!
// Let's see. resources.rs imports bevy_ecs. It typically doesn't depend on components.
// But components might depend on resources? No.
// Let's re-export them from crate root resources if the user wants `crate::components::WorkDone`.
pub use crate::resources::{AgentConcurrency, WorkDone};
