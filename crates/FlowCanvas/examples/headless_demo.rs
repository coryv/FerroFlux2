use flow_canvas::input::{InputState, MouseButtons};
use flow_canvas::model::GraphState;
use flow_canvas::{Canvas, CanvasConfig};
use glam::Vec2;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct MyNodeData {
    name: String,
    value: f32,
}

impl flow_canvas::model::NodeData for MyNodeData {
    fn node_type(&self) -> String {
        "HeadlessNode".to_string()
    }
}

fn main() {
    println!("=== FlowCanvas Headless Demo ===");

    // 1. Initialize Canvas
    let config = CanvasConfig::default();
    let mut canvas = Canvas::new(config);
    // Explicitly set a viewport size (simulating a window)
    canvas.update_viewport_size(Vec2::new(1280.0, 720.0));

    // 2. Initialize Graph State
    let mut graph = GraphState::<MyNodeData>::default();

    // 3. Populate Graph with some data
    let node1 = graph.insert_node(flow_canvas::model::Node {
        id: flow_canvas::model::NodeId::default(), // key replaced by helper
        uuid: uuid::Uuid::new_v4(),
        position: Vec2::new(100.0, 100.0),
        size: Vec2::new(150.0, 100.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        data: MyNodeData {
            name: "Node A".into(),
            value: 42.0,
        },
        flags: Default::default(),
        // Verify custom style: Red Node
        style: Some(flow_canvas::config::NodeStyle {
            color: glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
            border_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            text_color: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        }),
    });

    let node2 = graph.insert_node(flow_canvas::model::Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: uuid::Uuid::new_v4(),
        position: Vec2::new(400.0, 200.0),
        size: Vec2::new(150.0, 100.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        data: MyNodeData {
            name: "Node B".into(),
            value: 100.0,
        },
        flags: Default::default(),
        style: None, // Use default
    });

    println!("Created graph with 2 nodes:");
    println!("  - Node A: {:?}", graph.nodes.get(node1).map(|n| &n.data));
    println!("  - Node B: {:?}", graph.nodes.get(node2).map(|n| &n.data));

    // 4. Initialize History
    let mut history = flow_canvas::history::HistoryManager::<MyNodeData>::default();

    // 5. Simulate Simulation Loop
    for frame in 0..6 {
        println!("\n--- Frame {} ---", frame);

        // Simulation Logic
        if frame == 1 {
            println!(">> Committing state & Moving Node A...");
            history.commit(&graph);
            if let Some(node) = graph.nodes.get_mut(node1) {
                node.position += Vec2::new(50.0, 50.0);
            }
        } else if frame == 2 {
            println!(">> Committing state & Moving Node A again...");
            history.commit(&graph);
            if let Some(node) = graph.nodes.get_mut(node1) {
                node.position += Vec2::new(50.0, 50.0);
            }
        } else if frame == 3 {
            println!(">> Undoing last move...");
            if history.undo(&mut graph) {
                println!("   Undo successful!");
            } else {
                println!("   Undo failed!");
            }
        } else if frame == 4 {
            println!(">> Undoing initial move...");
            if history.undo(&mut graph) {
                println!("   Undo successful!");
            }
        } else if frame == 5 {
            println!(">> Redoing...");
            if history.redo(&mut graph) {
                println!("   Redo successful!");
            }
        }

        // Check Node A position
        if let Some(node) = graph.nodes.get(node1) {
            println!("  Node A Pos: {}", node.position);
        }

        // create fake input
        // Simulate mouse moving from (0,0) towards (200, 200)
        let t = frame as f32 / 10.0;
        let mouse_pos = Vec2::new(200.0 * t, 200.0 * t);

        let input = InputState {
            mouse_pos,
            mouse_buttons: MouseButtons::default(), // No clicks
            scroll_delta: 0.0,
            modifiers: Default::default(),
            pressed_keys: Vec::new(),
            screen_size: Vec2::new(1280.0, 720.0),
            event_consumed_by_content: false,
        };

        // Update Canvas
        let (_draw_list, events) = canvas.update(&input, 0.016, &mut graph);

        println!("  Logic Events: {:?}", events);
    }

    println!("\nDemo Complete.");
}
