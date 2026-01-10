use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::GraphState;
use futures::StreamExt;

/// A simple custom NodeData struct for our example.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MyData {
    pub label: String,
}

impl Default for MyData {
    fn default() -> Self {
        Self {
            label: "Node".to_string(),
        }
    }
}

impl flow_canvas::model::NodeData for MyData {
    fn node_type(&self) -> String {
        "MyData".to_string()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    // 1. Initialize SDK
    let (client, _handle) = FerroFluxClient::<MyData>::start().await?;

    // 2. Subscribe to logs and telemetry BEFORE execution
    let log_stream = client.logs();
    let telemetry_stream = client.telemetry();

    // 3. Spawn a task to print logs
    tokio::spawn(async move {
        tokio::pin!(log_stream);
        println!("--> Listening for Logs...");
        while let Some((level, msg)) = log_stream.next().await {
            println!("[LOG] [{}] {}", level, msg);
        }
    });

    // 4. Spawn a task to print telemetry
    tokio::spawn(async move {
        tokio::pin!(telemetry_stream);
        println!("--> Listening for Telemetry...");
        while let Some((node_id, success, ms, trace_id)) = telemetry_stream.next().await {
            println!(
                "[TELEMETRY] Node: {} | Success: {} | Time: {}ms | Trace: {}",
                node_id, success, ms, trace_id
            );
        }
    });

    // 5. Deploy empty graph (just to start ticking)
    let graph = GraphState::<MyData>::default();
    client.sync_graph(&graph).await?;

    println!("--> Engine running. Waiting 2 seconds to capture startup logs...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    println!("--> Shutting down.");
    Ok(())
}
