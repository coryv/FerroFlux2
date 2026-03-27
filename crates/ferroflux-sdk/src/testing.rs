use crate::FerroFluxClient;
use flow_canvas::model::{GraphState, Node, NodeData, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestScenario {
    pub name: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    pub blueprint: ferroflux_core::graph_loader::WorkflowBlueprint,
    pub steps: Vec<TestStep>,
}

fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum TestStep {
    #[serde(rename = "inject")]
    Inject {
        node: String, // Name or ID string
        value: serde_json::Value,
    },
    #[serde(rename = "wait")]
    Wait { duration_ms: u64 },
    #[serde(rename = "assert")]
    Assert {
        node: String,
        property: String,
        equals: serde_json::Value,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonNodeData(pub serde_json::Value);

impl NodeData for JsonNodeData {
    fn node_type(&self) -> String {
        self.0
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("Default")
            .to_string()
    }
}

pub async fn run_scenario(yaml: &str) -> anyhow::Result<()> {
    let scenario: TestScenario = serde_yaml::from_str(yaml)?;
    println!("--> Running Scenario: {}", scenario.name);

    // 1. Convert Blueprint to GraphState
    let mut graph = GraphState::<JsonNodeData>::default();
    let mut name_to_uuid = HashMap::new();

    // Layout Helper
    let mut x_cursor = 0.0;

    for node_bp in &scenario.blueprint.nodes {
        let uuid = node_bp.id;
        name_to_uuid.insert(node_bp.name.clone(), uuid);

        // Construct NodeData wrapper
        // We inject the "type" into data so NodeData::node_type works.
        // And we include the config.
        let mut data_val = node_bp.config.clone();
        if let Some(obj) = data_val.as_object_mut() {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String(node_bp.node_type.clone()),
            );
        }

        let node = Node {
            id: NodeId::default(),
            uuid,
            position: glam::Vec2::new(x_cursor, 0.0),
            size: glam::Vec2::new(150.0, 100.0),
            inputs: vec![],
            outputs: vec![],
            data: JsonNodeData(data_val),
            flags: Default::default(),
            style: None,
        };

        let _node_id = graph.insert_node(node);
        x_cursor += 200.0;
    }

    // Edges
    for edge_bp in &scenario.blueprint.edges {
        let src_node_id = *graph
            .uuid_index
            .get(&edge_bp.source_id)
            .ok_or_else(|| anyhow::anyhow!("Source node not found"))?;
        let tgt_node_id = *graph
            .uuid_index
            .get(&edge_bp.target_id)
            .ok_or_else(|| anyhow::anyhow!("Target node not found"))?;

        let src_port = graph.add_port(src_node_id, false); // Output
        let tgt_port = graph.add_port(tgt_node_id, true); // Input

        graph.connect(src_port, tgt_port);
    }

    // 2. Start Client
    let (client, _handle) = FerroFluxClient::<JsonNodeData>::start().await?;

    // 3. Deploy
    client.sync_graph(&graph).await?;

    // 4. Run Steps
    for step in scenario.steps {
        match step {
            TestStep::Wait { duration_ms } => {
                tokio::time::sleep(Duration::from_millis(duration_ms)).await;
            }
            TestStep::Inject { node, value } => {
                let uuid = if let Ok(u) = Uuid::parse_str(&node) {
                    u
                } else {
                    *name_to_uuid
                        .get(&node)
                        .ok_or_else(|| anyhow::anyhow!("Node name '{}' not found", node))?
                };

                println!("[Inject] {} -> {:?}", uuid, value);
                client.inject_message(uuid, "default", value).await?;
            }
            TestStep::Assert {
                node,
                property,
                equals,
            } => {
                let uuid = if let Ok(u) = Uuid::parse_str(&node) {
                    u
                } else {
                    *name_to_uuid
                        .get(&node)
                        .ok_or_else(|| anyhow::anyhow!("Node name '{}' not found", node))?
                };

                let state = client
                    .inspect_node(uuid)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Node {} state not found", uuid))?;

                if property.starts_with("outbox.last") {
                    if let Some((_, ticket)) = state.outbox.queue.back() {
                        let blob = client.read_blob(ticket.clone()).await?.ok_or_else(|| {
                            anyhow::anyhow!("Blob not found for ticket {:?}", ticket)
                        })?;

                        let mut current = blob;

                        let parts: Vec<&str> = property.split('.').collect();
                        if parts.len() > 2 {
                            for part in &parts[2..] {
                                if let Some(val) = current.get(part) {
                                    current = val.clone();
                                } else {
                                    current = serde_json::Value::Null;
                                    break;
                                }
                            }
                        }

                        if current != equals {
                            return Err(anyhow::anyhow!(
                                "Expected output {:?}, got {:?}",
                                equals,
                                current
                            ));
                        }
                    } else if equals != serde_json::Value::Null {
                        return Err(anyhow::anyhow!("Expected output {:?}, got None", equals));
                    }
                } else if property == "inbox.count" {
                    let count = state.inbox.queue.len();
                    if serde_json::json!(count) != equals {
                        return Err(anyhow::anyhow!(
                            "Expected inbox count {}, got {}",
                            equals,
                            count
                        ));
                    }
                }

                println!("[Assert] Passed");
            }
        }
    }

    Ok(())
}
