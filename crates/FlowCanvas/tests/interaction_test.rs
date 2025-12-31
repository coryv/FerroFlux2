use flow_canvas::{
    Canvas, CanvasConfig, InteractionMode,
    input::InputState,
    model::{GraphState, Node, NodeFlags},
};
use glam::Vec2;

fn create_test_graph() -> (GraphState<String>, flow_canvas::model::NodeId) {
    let mut graph = GraphState::default();
    let id = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: Vec2::new(100.0, 100.0),
        size: Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: "Test Node".to_string(),
        flags: NodeFlags::default(),
        style: None,
    });
    graph.nodes[id].id = id;
    // Important: populate draw_order
    graph.draw_order.push(id);
    (graph, id)
}

#[test]
fn test_panning() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let mut graph: GraphState<String> = GraphState::default(); // Empty graph

    // 1. Initial State
    assert_eq!(canvas.view.transform.pan, Vec2::ZERO);

    // 2. Start Pan (Middle Click)
    let mut input = InputState {
        mouse_pos: Vec2::new(100.0, 100.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            middle: true,
            ..Default::default()
        },
        ..Default::default()
    };

    // Update should transition to Panning
    canvas.update(&input, 0.016, &mut graph);

    match canvas.interaction_mode {
        InteractionMode::Panning { .. } => {} // OK
        _ => panic!("Should be in Panning state"),
    }

    // 3. Move Mouse (Still Middle Click)
    input.mouse_pos = Vec2::new(150.0, 120.0); // +50, +20
    canvas.update(&input, 0.016, &mut graph);

    // Pan should have updated
    assert_eq!(canvas.view.transform.pan, Vec2::new(50.0, 20.0));

    // 4. Release Middle Click
    input.mouse_buttons.middle = false;
    canvas.update(&input, 0.016, &mut graph);

    match canvas.interaction_mode {
        InteractionMode::Idle => {} // OK
        _ => panic!("Should return to Idle"),
    }
}

#[test]
fn test_selection_and_z_ordering() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let (mut graph, node_id) = create_test_graph();

    // Create a second node which overlaps/is behind
    let node2_id = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: Vec2::new(150.0, 150.0),
        size: Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: "Node 2".to_string(),
        flags: NodeFlags::default(),
        style: None,
    });
    graph.nodes[node2_id].id = node2_id;
    graph.draw_order.push(node2_id); // draw_order: [node_id, node2_id]

    assert_eq!(graph.draw_order.len(), 2);
    assert_eq!(graph.draw_order[0], node_id);
    assert_eq!(graph.draw_order[1], node2_id);

    // 1. Click Node 1 (at 110, 110) - Default pan
    let input = InputState {
        mouse_pos: Vec2::new(110.0, 110.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: true,
            ..Default::default()
        },
        ..Default::default()
    };

    canvas.update(&input, 0.016, &mut graph);

    // Node 1 should be selected
    let node1 = &graph.nodes[node_id];
    assert!(node1.flags.contains(NodeFlags::SELECTED));

    // Node 2 should NOT be selected
    let node2 = &graph.nodes[node2_id];
    assert!(!node2.flags.contains(NodeFlags::SELECTED));

    // Node 1 should be moved to END of draw_order (Front)
    // draw_order: [node2_id, node_id]
    assert_eq!(graph.draw_order[0], node2_id);
    assert_eq!(graph.draw_order[1], node_id);

    match canvas.interaction_mode {
        InteractionMode::DraggingNodes { .. } => {}
        _ => panic!("Should be dragging"),
    }
}

#[test]
fn test_dragging() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let (mut graph, node_id) = create_test_graph();

    // 1. Click on Node to start drag
    let mut input = InputState {
        mouse_pos: Vec2::new(110.0, 110.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: true,
            ..Default::default()
        },
        ..Default::default()
    };
    canvas.update(&input, 0.016, &mut graph);

    // 2. Drag
    input.mouse_pos = Vec2::new(120.0, 120.0); // +10, +10 delta
    canvas.update(&input, 0.016, &mut graph);

    // ...
    let node = &graph.nodes[node_id];
    assert_eq!(node.position, Vec2::new(110.0, 110.0));
}

use flow_canvas::LogicEvent;

#[test]
fn test_linking() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let mut graph = GraphState::<()>::default();

    // Create Node A with Output
    let node_a = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: glam::Vec2::new(0.0, 0.0),
        size: glam::Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: (),
        flags: flow_canvas::model::NodeFlags::empty(),
        style: None,
    });
    // Fixup ID
    graph.nodes[node_a].id = node_a;

    // Create Output Port for A
    let port_out = graph.ports.insert(flow_canvas::model::Port {
        id: flow_canvas::model::PortId::default(),
        node: node_a,
    });
    graph.ports[port_out].id = port_out;
    graph.nodes[node_a].outputs.push(port_out);

    // Create Node B with Input
    let node_b = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: glam::Vec2::new(200.0, 0.0),
        size: glam::Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: (),
        flags: flow_canvas::model::NodeFlags::empty(),
        style: None,
    });
    graph.nodes[node_b].id = node_b;

    let port_in = graph.ports.insert(flow_canvas::model::Port {
        id: flow_canvas::model::PortId::default(),
        node: node_b,
    });
    graph.ports[port_in].id = port_in;
    graph.nodes[node_b].inputs.push(port_in);

    // Populate Draw Order
    graph.draw_order.push(node_a);
    graph.draw_order.push(node_b);

    // 1. Click on Output Port of A
    // Node A is at 0,0 size 100,100.
    // Output port is at (100.0, 50.0)

    let input_click = InputState {
        mouse_pos: glam::Vec2::new(100.0, 50.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let (_, events) = canvas.update(&input_click, 0.016, &mut graph);
    assert!(events.is_empty());

    match canvas.interaction_mode {
        InteractionMode::Linking { source, .. } => {
            assert_eq!(source, port_out);
        }
        _ => panic!("Should be in Linking state"),
    }

    // 2. Drag to Input Port of B
    // Node B at 200,0.
    // Input port at (200.0, 50.0).
    // Let's drag exactly there.
    let input_drag = InputState {
        mouse_pos: glam::Vec2::new(200.0, 50.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: true,
            ..Default::default()
        },
        ..Default::default()
    };
    canvas.update(&input_drag, 0.016, &mut graph);

    // 3. Release
    let input_release = InputState {
        mouse_pos: glam::Vec2::new(200.0, 50.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let (_, events) = canvas.update(&input_release, 0.016, &mut graph);

    // Should have a Connect event and RepaintNeeded(s)
    // 1. RepaintNeeded (Normal update loop: wire position changed)
    // 2. Connect
    // 3. RepaintNeeded (Because logic changed)
    assert_eq!(events.len(), 3);

    // Find Connect event
    let connect_event = events
        .iter()
        .find(|e| matches!(e, LogicEvent::Connect { .. }));
    assert!(connect_event.is_some());

    match connect_event.unwrap() {
        LogicEvent::Connect { from, to } => {
            assert_eq!(*from, port_out);
            assert_eq!(*to, port_in);
        }
        _ => panic!("Expected Connect event"),
    }

    // Should return to Idle
    match canvas.interaction_mode {
        InteractionMode::Idle => {}
        _ => panic!("Should be Idle"),
    }
}

#[test]
fn test_box_selection() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let mut graph = GraphState::<()>::default();

    // Create Node A at 0,0 size 100,100
    let node_a = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: glam::Vec2::new(0.0, 0.0),
        size: glam::Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: (),
        flags: flow_canvas::model::NodeFlags::empty(),
        style: None,
    });
    graph.nodes[node_a].id = node_a;

    // Create Node B at 200,200 size 100,100
    let node_b = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: glam::Vec2::new(200.0, 200.0),
        size: glam::Vec2::new(100.0, 100.0),
        inputs: vec![],
        outputs: vec![],
        data: (),
        flags: flow_canvas::model::NodeFlags::empty(),
        style: None,
    });
    graph.nodes[node_b].id = node_b;

    // 1. Start Box Select at -50, -50 (Top Left of A)
    let input_start = InputState {
        mouse_pos: glam::Vec2::new(-50.0, -50.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: true,
            ..Default::default()
        },
        ..Default::default()
    };

    canvas.update(&input_start, 0.016, &mut graph);
    match canvas.interaction_mode {
        InteractionMode::BoxSelecting { .. } => {}
        _ => panic!("Should be BoxSelecting"),
    }

    // 2. Drag to 150, 150 (Bottom Right of A, but not covering B)
    let input_drag = InputState {
        mouse_pos: glam::Vec2::new(150.0, 150.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: true,
            ..Default::default()
        },
        ..Default::default()
    };
    canvas.update(&input_drag, 0.016, &mut graph);

    // 3. Release
    let input_release = InputState {
        mouse_pos: glam::Vec2::new(150.0, 150.0),
        mouse_buttons: flow_canvas::input::MouseButtons {
            left: false,
            ..Default::default()
        },
        ..Default::default()
    };
    canvas.update(&input_release, 0.016, &mut graph);

    // Verify Node A is Selected, Node B is NOT
    assert!(
        graph.nodes[node_a]
            .flags
            .contains(flow_canvas::model::NodeFlags::SELECTED)
    );
    assert!(
        !graph.nodes[node_b]
            .flags
            .contains(flow_canvas::model::NodeFlags::SELECTED)
    );
}

#[test]
fn test_shortcuts() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let (mut graph, node_id) = create_test_graph();

    // 1. Select the node manually
    graph.nodes[node_id].flags.insert(NodeFlags::SELECTED);

    // 2. Press Delete
    let input_delete = InputState {
        pressed_keys: vec![flow_canvas::input::Key::Delete],
        ..Default::default()
    };

    let (_, events) = canvas.update(&input_delete, 0.016, &mut graph);

    // Should emit DeleteSelection and RepaintNeeded
    assert_eq!(events.len(), 2);
    match events[0] {
        LogicEvent::DeleteSelection => {} // OK
        _ => panic!("Expected DeleteSelection"),
    }
    assert_eq!(events[1], LogicEvent::RepaintNeeded);

    // 3. Select All
    // Deselect first
    graph.nodes[node_id].flags.remove(NodeFlags::SELECTED);

    let input_select_all = InputState {
        pressed_keys: vec![flow_canvas::input::Key::A],
        modifiers: flow_canvas::input::ModifiersState {
            ctrl: true, // or meta
            ..Default::default()
        },
        ..Default::default()
    };

    canvas.update(&input_select_all, 0.016, &mut graph);

    // Node should be selected
    assert!(graph.nodes[node_id].flags.contains(NodeFlags::SELECTED));
}

#[test]
fn test_zooming() {
    let mut canvas = Canvas::new(CanvasConfig::default());
    let mut graph = GraphState::<()>::default();

    // Initial State: Zoom 1.0, Pan 0,0
    assert_eq!(canvas.view.transform.zoom, 1.0);
    assert_eq!(canvas.view.transform.pan, Vec2::ZERO);

    // Mouse at (100, 100). World at (100, 100).
    // We zoom IN (scroll +1.0). Config zoom speed is 0.1.
    // Factor = 1.1. New Zoom = 1.1.
    // Expected behavior: World(100,100) should still be at Screen(100,100).

    let input = InputState {
        mouse_pos: Vec2::new(100.0, 100.0),
        scroll_delta: 1.0,
        ..Default::default()
    };

    canvas.update(&input, 0.016, &mut graph);

    // Check Zoom
    assert!((canvas.view.transform.zoom - 1.1).abs() < 0.001);

    // Check Stability
    let world_under_mouse = canvas.view.screen_to_world(Vec2::new(100.0, 100.0));
    assert!((world_under_mouse.x - 100.0).abs() < 0.001);
    assert!((world_under_mouse.y - 100.0).abs() < 0.001);

    // Check Pan
    // Pan = Screen - World * Zoom = 100 - (100 * 1.1) = 100 - 110 = -10.0
    assert!((canvas.view.transform.pan.x - -10.0).abs() < 0.001);
    assert!((canvas.view.transform.pan.y - -10.0).abs() < 0.001);
}
