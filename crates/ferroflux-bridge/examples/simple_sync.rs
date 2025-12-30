use ferroflux_bridge::FerroFluxAdapter;
use ferroflux_core::app::AppBuilder;
use flow_canvas::model::GraphState;
use glam::Vec2;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct MyData;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!("=== FerroFlux Adapter Integration Demo ===");

    // 1. Initialize the FerroFlux Engine
    let (engine, _api_tx, event_tx, ..) = AppBuilder::new().build().await?;

    // 2. Initialize the Bridge Adapter
    let mut adapter = FerroFluxAdapter::<MyData>::new(engine, event_tx);

    // 3. Create a Visual Graph in FlowCanvas
    let mut graph = GraphState::<MyData>::default();

    // Add Node A
    let _node_a_id = graph.nodes.insert(flow_canvas::model::Node {
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

    // Add Node B
    let _node_b_id = graph.nodes.insert(flow_canvas::model::Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: Uuid::new_v4(),
        position: Vec2::new(400.0, 100.0),
        size: Vec2::new(150.0, 100.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        data: MyData,
        flags: Default::default(),
        style: None,
    });

    println!("Canvas created with 2 nodes.");

    // 4. Deploy to Engine
    println!("Deploying canvas to engine...");
    adapter.deploy(&graph).await?;

    // 5. Run a few ticks of the engine
    println!("Running engine ticks...");
    for i in 0..5 {
        adapter.tick().await?;
        adapter.sync_events(&mut graph);
        println!("Tick {} complete", i);
    }

    println!("\nIntegration Demo Succeeded!");
    Ok(())
}
