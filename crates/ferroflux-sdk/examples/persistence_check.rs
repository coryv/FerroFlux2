use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::{GraphState, NodeData};

/// Custom NodeData
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MyData {
    pub label: String,
}

impl Default for MyData {
    fn default() -> Self {
        Self {
            label: "Persisted Node".to_string(),
        }
    }
}

impl NodeData for MyData {
    fn node_type(&self) -> String {
        "MyData".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    // 1. Start Client 1
    println!("--> Starting Client 1");
    let (client1, _handle1) = FerroFluxClient::<MyData>::start().await?;

    // 2. Deploy a Graph
    let mut graph = GraphState::<MyData>::default();
    let node_id = uuid::Uuid::new_v4();
    let node = flow_canvas::model::Node {
        id: Default::default(),
        uuid: node_id,
        position: glam::Vec2::ZERO,
        size: glam::Vec2::new(150.0, 100.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        data: MyData::default(),
        flags: Default::default(),
        style: None,
    };
    graph.insert_node(node);

    println!("--> Syncing Graph to Client 1");
    client1.sync_graph(&graph).await?;

    // 3. Save Snapshot
    println!("--> Saving Snapshot");
    let json = client1.save_snapshot(&graph).await?;
    println!("--> Snapshot Size: {} bytes", json.len());
    println!("--> Content: {}", json);

    // 4. Start Client 2 (New Engine)
    println!("--> Starting Client 2");
    let (client2, _handle2) = FerroFluxClient::<MyData>::start().await?;

    // 5. Load Snapshot
    println!("--> Loading Snapshot into Client 2");
    client2.load_snapshot(&json).await?;

    println!("--> Snapshot loaded successfully. Verifying liveness...");
    client2.resume().await?; // Just to poke it

    println!("--> Done.");
    Ok(())
}
