//! # ferroflux-integration
//!
//! Parsing, loading, and semantic validation of FerroFlux YAML integration files
//! (platform definitions and action node definitions).
//!
//! This crate has no runtime or ECS dependencies — it is usable standalone,
//! as a library inside `ferroflux_core`, or via the `ferroflux-validate` CLI.

pub mod definition;
pub mod hooks;
pub mod registry;
pub mod validation;

// Convenience re-exports
pub use definition::{
    Interface, NodeDefinition, NodeMeta, PipelineStep, PlatformDefinition, PortDef,
    PortGeneratorDef, PortTemplateDef, RoutingAction, RoutingLogic, SettingDef,
};
pub use hooks::{LoadHooks, NoopHooks};
pub use registry::DefinitionRegistry;
pub use validation::{
    validate_cross, validate_node, validate_platform, Severity, ValidationDiagnostic,
    ValidationResult,
};
