# FerroFlux SDK

The **FerroFlux SDK** provides a high-level Rust interface for embedding the **FerroFlux Engine** into your applications. It bridges the gap between the visual graph representation (`FlowCanvas`) and the execution runtime (`FerroFlux-core`).

## Architecture: Actor Model

As of v0.2.0, the SDK uses an **Actor Model**.
- The **Engine** runs in a dedicated background task (`EngineActor`), ensuring it ticks at a consistent rate effectively decoupling it from your main application loop.
- The **Client** (`FerroFluxClient`) communicates with the engine via async channels.

## Quick Start

Add `ferroflux-sdk` to your `Cargo.toml`. Then, initialize the client and start deployment:

```rust
use ferroflux_sdk::FerroFluxClient;
use flow_canvas::model::GraphState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Start the SDK (Spawns the Engine Actor in background)
    let (client, _handle) = FerroFluxClient::<MyNodeData>::start().await?;

    // 2. Create your visual graph (using flow_canvas)
    let mut graph = GraphState::default();
    // ... add nodes ...

    // 3. Deploy to Engine (Incremental Sync)
    // This updates the running engine to match your graph without stopping execution.
    client.sync_graph(&graph).await?;

    // 4. Control Execution
    client.pause().await?;
    client.resume().await?;

    // 5. Visualize (Optional)
    // Poll for execution events to animate your UI
    loop {
        client.sync_events(&mut graph);
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }
}
```

## Key Features

### Live Editing (`sync_graph`)
Unlike previous versions which required a full reset, `sync_graph` performs an **incremental reconciliation**:
- New nodes in the graph are spawned.
- Removed nodes are despawned.
- Existing nodes (same UUID) are preserved, maintaining their runtime state (memory, inbox, etc.).

This allows you to modify the graph logic *while signals are flowing through it*.

### Execution Control
- **`pause()`**: Suspends the engine's update loop. The Actor remains responsive to commands.
- **`resume()`**: Resumes the update loop.
- **`step(n)`**: (While paused) Executes exactly `n` ticks of the engine.

### Shadow Mode Simulation
Use `simulate_and_wait` to run a specific node in isolation (mocking inputs/outputs) without affecting the main graph flow.

```rust
let result = client.simulate_and_wait(
    node_id,
    "my.node.def",
    config,
    input_payload,
    mock_config
).await?;
```
