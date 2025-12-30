# FlowCanvas

**FlowCanvas** is a headless, highly optimized, retained-mode graph library for Rust.
It acts as the "operating system" for node graphs, handling state, mathematics, interactions, and logic, while delegating the actual rendering to your host application (e.g., `wgpu`, `pixels`, `macroquad`, or even a terminal).

## Features

- **Headless Architecture**: You bring the renderer, we handle the logic.
- **Infinite Canvas**: Built-in Pan & Zoom coordinate systems.
- **Interactions**: Drag-and-drop, box selection, linking, port snapping.
- **Robust Persistence**: Save/Load using stable UUIDs.
- **Undo/Redo**: Full state snapshot history system.
- **Optimized Model**: Uses `SlotMap` for O(1) lookups and cache-friendly data structures.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
flow_canvas = { path = "." } # Or git/crates.io version
glam = "0.24" # For vector math
uuid = { version = "1.0", features = ["v4", "serde"] }
```

## Quickstart

Here is how to get a basic graph up and running in 4 steps.

### 1. Define Node Data

Define the payload that each node will carry.

```rust
use flow_canvas::model::NodeData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MyNodeData {
    title: String,
    value: f32,
}
```

### 2. Initialize Canvas & Graph

Create the transient editor state (`Canvas`) and the persistent data state (`GraphState`).

```rust
use flow_canvas::{Canvas, CanvasConfig};
use flow_canvas::model::{GraphState, Node, NodeId, Uuid};
use glam::Vec2;

// Setup
let config = CanvasConfig::default();
let mut canvas = Canvas::new(config);
let mut graph = GraphState::<MyNodeData>::default();

// Create a Node
let node_id = graph.nodes.insert(Node {
    id: NodeId::default(), // Placeholder, SlotMap overwrites this key
    uuid: Uuid::new_v4(),  // Stable ID for persistence
    position: Vec2::new(100.0, 100.0),
    size: Vec2::new(150.0, 80.0),
    inputs: vec![],
    outputs: vec![],
    data: MyNodeData { title: "Start".into(), value: 0.0 },
    flags: Default::default(),
});
// Fixup self-reference
graph.nodes[node_id].id = node_id;
graph.draw_order.push(node_id);
```

### 3. The Update Loop

In your application's main loop, feed `InputState` into the `canvas.update()` method.

```rust
use flow_canvas::input::{InputState, MouseButtons};

// In your event loop...
let input = InputState {
    mouse_pos: Vec2::new(150.0, 150.0), // From your windowing system
    mouse_buttons: MouseButtons { left: true, ..Default::default() },
    ..Default::default()
};

// Update
// dt is delta time in seconds
let (render_list, events) = canvas.update(&input, 0.016, &mut graph);
```

### 4. Handle Output

- **Render**: Iterate through `render_list` and draw the primitives (Rects, Lines, BezierCurves).
- **Events**: Process `events` (e.g., `LogicEvent::Connect`, `LogicEvent::DeleteSelection`).

```rust
for command in render_list {
    match command {
        flow_canvas::render::DrawCommand::Rect { pos, size, color, .. } => {
            // draw_rect(pos, size, color);
        }
        flow_canvas::render::DrawCommand::BezierCubic { p0, p1, p2, p3, .. } => {
             // draw_curve(p0, p1, p2, p3);
        }
        _ => {}
    }
}

for event in events {
    match event {
        flow_canvas::LogicEvent::Connect { from, to } => {
            println!("Connected port {:?} to {:?}", from, to);
             // Verify connection rules here, then manually add connection:
             // graph.connections.insert(...)
        }
        _ => {}
    }
}
```

## Running the Demo

To see a headless simulation of the library in action, run:

```bash
cargo run --example headless_demo
```
