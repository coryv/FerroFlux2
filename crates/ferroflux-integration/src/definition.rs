use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Metadata for AI agents to discover and use this node correctly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMetadata {
    /// High-level purpose of the node for agent reasoning.
    pub purpose: String,
    /// Detailed context on when to use this action over others.
    pub usage_context: String,
    /// Specific hints or "gotchas" for the agent.
    #[serde(default)]
    pub agent_hints: Vec<String>,
    /// Representative WAML examples for this node.
    #[serde(default)]
    pub waml_examples: Vec<WamlExample>,
}

/// A WAML example for documentation and agent guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WamlExample {
    /// Short title for the example.
    pub title: String,
    /// The WAML snippet itself.
    pub waml: String,
}

/// The top-level structure of a YAML Node Definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub meta: NodeMeta,
    pub interface: Interface,
    pub context: Option<HashMap<String, String>>, // Mappings: "key" -> "{{ settings.val }}"
    pub execution: Vec<PipelineStep>,
    pub output_transform: Option<HashMap<String, String>>,
    pub routing: Option<RoutingLogic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDefinition {
    pub meta: NodeMeta,
    pub config: HashMap<String, Value>,
    #[serde(default)]
    pub settings: Vec<SettingDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMeta {
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(rename = "type")]
    pub node_type: String, // Action, Trigger, Utility, etc.
    pub description: Option<String>,
    pub version: Option<String>,
    pub platform: Option<String>,
    pub data_strategy: Option<String>, // enrich, replace, split, aggregate
    /// Structural subtype for nodes that deviate from the standard Exec→Success/Error pattern.
    /// Recognised values: Router, Iterator, Accumulator, Terminus
    pub node_subtype: Option<String>,
    /// Cryptographic signature for verifying the integrity and authorship of the integration.
    pub signature: Option<ferroflux_security::signing::IntegrationSignature>,
    /// Explicitly declared permissions/capabilities required by this node (e.g. "network:api.slack.com").
    #[serde(default)]
    pub permissions: Vec<String>,
    /// AI Agent discovery and usage guidance.
    pub discovery: Option<DiscoveryMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub inputs: Vec<PortDef>,
    pub outputs: Vec<PortDef>,
    #[serde(default)]
    pub settings: Vec<SettingDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    #[serde(default)]
    pub default_hidden: bool,
    #[serde(default)]
    pub generator: Option<PortGeneratorDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortGeneratorDef {
    pub source: String,
    pub template: PortTemplateDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTemplateDef {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDef {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub default: Option<Value>,
    pub required: Option<bool>,
    pub options: Option<Value>,
    pub options_provider: Option<String>,
    pub read_only: Option<bool>,
    pub placeholder: Option<String>,
    pub show_if: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: String,
    pub tool: String,  // Tool ID
    pub params: Value, // Template string values allowed
    #[serde(default)]
    pub returns: HashMap<String, String>, // Method output -> context key
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingLogic {
    #[serde(rename = "match", default = "default_match")]
    pub match_expr: String,
    pub cases: HashMap<String, Vec<RoutingAction>>,
}

fn default_match() -> String {
    "success".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAction {
    pub tool: String,
    pub params: Value,
    #[serde(default)]
    pub returns: HashMap<String, String>,
}
