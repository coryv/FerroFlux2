use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::GraphState;
use glam::Vec2;
use uuid::Uuid;

use futures::StreamExt;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct MyData;

impl flow_canvas::model::NodeData for MyData {
    fn node_type(&self) -> String {
        "DebugNode".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!("=== FerroFlux SDK Integration Demo (Actor Mode) ===");

    // 1. Initialize the SDK (which starts the background Actor)
    let (client, actor_handle) = FerroFluxClient::<MyData>::start().await?;

    // 2. Spawn a log listener
    let log_stream = client.logs();
    tokio::spawn(async move {
        tokio::pin!(log_stream);
        while let Some((level, msg)) = log_stream.next().await {
            println!("[LOG] [{}] {}", level, msg);
        }
    });

    // 3. Create a Visual Graph in FlowCanvas
    let mut graph = GraphState::<MyData>::default();

    // Add Node A
    let _node_a_id = graph.insert_node(flow_canvas::model::Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: Uuid::new_v4(),
        position: Vec2::new(100.0, 100.0),
        size: Vec2::new(150.0, 100.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        data: MyData,
        flags: Default::default(),
        style: None,
    });

    println!("Canvas created with 1 node.");

    // 4. Deploy to Engine (Incremental Sync)
    println!("Deploying canvas to engine...");
    client.sync_graph(&graph).await?;

    // 5. Let it run for a bit
    println!("Engine is running in background...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 6. Live Edit: Add a new node
    println!("Live Edit: Adding Node B...");
    let _node_b_id = graph.insert_node(flow_canvas::model::Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: Uuid::new_v4(), // New UUID
        position: Vec2::new(400.0, 100.0),
        size: Vec2::new(150.0, 100.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        data: MyData,
        flags: Default::default(),
        style: None,
    });

    client.sync_graph(&graph).await?;
    println!("Synced new graph state.");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 7. Pause/Resume
    println!("Pausing engine...");
    client.pause().await?;

    println!("Stepping 1 frame...");
    client.step(1).await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("Resuming engine...");
    client.resume().await?;

    // Cleanup
    // In a real app we might want a Shutdown command, but dropping the client closes the channel
    // and the Actor loop will exit.
    drop(client);
    let _ = actor_handle.await;

    println!("\nIntegration Demo Succeeded!");
    Ok(())
}
