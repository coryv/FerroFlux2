use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Schema Negotiation: What fields this node *needs* from upstream.
///
/// Used during validation to ensure the graph is wired correctly.
#[derive(Component, Debug, Clone, Default)]
pub struct Requirements {
    /// Set of JSON-path like field names required.
    pub needed_fields: HashSet<String>,
}

/// Schema Negotiation: What fields this node *promises* to produce.
///
/// Used during validation and to populate autocomplete in the UI specific to downstream nodes.
#[derive(Component, Debug, Clone, Default)]
pub struct ExpectedOutput {
    /// Set of fields guaranteed to be present in the output.
    pub aggregated_schema: HashSet<String>,
}
