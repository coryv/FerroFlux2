use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// The top-level structure of a YAML Node Definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    pub meta: NodeMeta,
    pub interface: Interface,
    pub context: Option<HashMap<String, String>>, // Mappings: "key" -> "{{ settings.val }}"
    pub execution: Vec<PipelineStep>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDef {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub default: Option<Value>,
    pub required: Option<bool>,
    pub options_provider: Option<String>,
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
    #[serde(rename = "match")]
    pub match_expr: String,
    pub cases: HashMap<String, Vec<RoutingAction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAction {
    pub tool: String,
    pub params: Value,
    #[serde(default)]
    pub returns: HashMap<String, String>,
}
