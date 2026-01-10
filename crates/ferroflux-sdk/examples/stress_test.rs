use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::{GraphState, Node, NodeId};
use glam::Vec2;
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct MyData;
impl flow_canvas::model::NodeData for MyData {
    fn node_type(&self) -> String {
        "core.action.log".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt::init(); // Disable logging for speed

    println!("--> Starting Stress Test: High Node Count & Message Throughput");

    let (client, _handle) = FerroFluxClient::<MyData>::start().await?;
    let mut graph = GraphState::<MyData>::default();

    // 1. Build a Graph: 100 Parallel Chains of 10 Nodes (1000 nodes total)
    let chains = 100;
    let depth = 10;
    let mut total_nodes = 0;

    let mut start_nodes = Vec::new();
    let mut end_nodes = Vec::new();

    println!("Building graph ({} chains x {} depth)...", chains, depth);
    for c in 0..chains {
        let mut prev_port = None;
        let mut first_node_uuid = None;

        for d in 0..depth {
            let uuid = Uuid::new_v4();
            let node = Node {
                id: NodeId::default(),
                uuid,
                position: Vec2::ZERO, // Layout doesn't matter
                size: Vec2::ONE,
                inputs: vec![],
                outputs: vec![],
                data: MyData,
                flags: Default::default(),
                style: None,
            };
            let node_id = graph.insert_node(node);
            total_nodes += 1;

            if d == 0 {
                first_node_uuid = Some(uuid);
            }
            if d == depth - 1 {
                end_nodes.push(uuid);
            }

            // Connect to previous
            let input_port = graph.add_port(node_id, true);
            let output_port = graph.add_port(node_id, false);

            if let Some(prev) = prev_port {
                graph.connect(prev, input_port);
            }
            prev_port = Some(output_port);
        }
        start_nodes.push(first_node_uuid.unwrap());
    }

    println!("Graph built. Syncing...");
    let start = Instant::now();
    client.sync_graph(&graph).await?;
    println!("Sync complete in {:?}", start.elapsed());

    // 2. Inject Messages
    println!("Injecting 1 message into each of {} chains...", chains);
    let start_inject = Instant::now();
    for (i, uuid) in start_nodes.iter().enumerate() {
        let payload = serde_json::json!({ "chain": i, "val": 1 });
        client.inject_message(*uuid, "default", payload).await?;
    }
    println!("Injection complete in {:?}", start_inject.elapsed());

    // 3. Monitor for completion
    // Since we don't have a "Global Done" event, we poll the end nodes?
    // Or we rely on telemetry stream to count "Completed" executions.
    // Telemetry stream is better for aggregate stats.

    println!("Monitoring telemetry...");
    let mut completed_executions = 0;
    let expected_executions = chains * depth; // Each message travels 10 hops

    // Subscribe to telemetry
    let stream = client.telemetry(); // Assuming we exposed this or similiar
    // Wait, client.telemetry() returns a stream.

    use futures::StreamExt;
    tokio::pin!(stream);

    let monitor_start = Instant::now();
    while let Some(event) = stream.next().await {
        // Count successes
        completed_executions += 1;
        if completed_executions % 100 == 0 {
            print!(
                "\rProgress: {}/{}",
                completed_executions, expected_executions
            );
            use std::io::Write;
            std::io::stdout().flush()?;
        }

        if completed_executions >= expected_executions {
            break;
        }

        // Timeout
        if monitor_start.elapsed().as_secs() > 10 {
            println!("\nTimeout waiting for completion!");
            break;
        }
    }

    println!("\nDone. Total Time: {:?}", monitor_start.elapsed());
    let rps = (expected_executions as f64) / monitor_start.elapsed().as_secs_f64();
    println!("Throughput: {:.2} node-executions/sec", rps);

    Ok(())
}
