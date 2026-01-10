use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::GraphState;
use flow_canvas::model::NodeData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct MyData;

impl NodeData for MyData {
    fn node_type(&self) -> String {
        "Default".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (client, _handle) = FerroFluxClient::<MyData>::start().await?;

    let mut graph = GraphState::<MyData>::default();
    let node_id = uuid::Uuid::new_v4();
    let node = flow_canvas::model::Node {
        id: Default::default(),
        uuid: node_id,
        position: glam::Vec2::ZERO,
        size: glam::Vec2::ONE,
        inputs: vec![],
        outputs: vec![],
        data: MyData,
        flags: Default::default(),
        style: None,
    };
    graph.insert_node(node);

    client.sync_graph(&graph).await?;

    // Inspect
    println!("Inspecting Node: {}", node_id);
    let state = client.inspect_node(node_id).await?;

    match state {
        Some(s) => {
            println!("Node Found!");
            println!("Inbox Size: {}", s.inbox.queue.len());
            println!("Outbox Size: {}", s.outbox.queue.len());
            assert_eq!(s.inbox.queue.len(), 0);
        }
        None => {
            println!("Node Not Found!");
            panic!("Should have found the node");
        }
    }

    println!("Inspection Verified.");
    Ok(())
}
