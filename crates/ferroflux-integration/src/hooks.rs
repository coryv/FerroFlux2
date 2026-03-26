use crate::definition::{NodeDefinition, PlatformDefinition};
use crate::validation::ValidationDiagnostic;

/// Callbacks invoked during `DefinitionRegistry` loading.
///
/// Implement this trait to react to load events — e.g. to bridge loaded
/// definitions into a Bevy ECS world — without this crate needing to know
/// about Bevy or any other runtime.
pub trait LoadHooks {
    fn on_platform_loaded(&mut self, _id: &str, _def: &PlatformDefinition) {}
    fn on_node_loaded(&mut self, _id: &str, _def: &NodeDefinition) {}
    fn on_validation_error(&mut self, _diag: &ValidationDiagnostic) {}
}

/// No-op implementation used when no hooks are needed.
pub struct NoopHooks;
impl LoadHooks for NoopHooks {}
