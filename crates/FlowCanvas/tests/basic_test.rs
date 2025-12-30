use flow_canvas::{
    Canvas, CanvasConfig,
    model::{GraphState, Node, NodeFlags},
};
use glam::Vec2;

#[test]
fn test_basic_rendering() {
    // 1. Setup Graph
    use flow_canvas::input::InputState;

    // ...

    // 1. Setup Graph
    let mut graph: GraphState<String> = GraphState::default();

    let node_id = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: flow_canvas::model::Uuid::new_v4(),
        position: Vec2::new(100.0, 100.0),
        size: Vec2::new(100.0, 50.0),
        inputs: vec![],
        outputs: vec![],
        data: "Test Node".to_string(),
        flags: NodeFlags::default(),
        style: None,
    });
    // Update the self-reference ID
    graph.nodes[node_id].id = node_id;

    // 2. Setup Canvas
    let config = CanvasConfig::default();
    let mut canvas = Canvas::new(config);
    let input = InputState::default();

    // 3. Update
    let (draw_list, _events) = canvas.update(&input, 0.016, &mut graph);

    // 4. Verify
    assert!(!draw_list.is_empty(), "Draw list should not be empty");

    // Check if we have a Rect at the expected position
    // Default pan is (0,0), zoom is 1.0.
    // Node is at (100, 100).
    // Expected screen pos = (100 * 1.0) + 0 = 100.

    // Find the Rect command (ignore Grid Lines)
    let rect_cmd = draw_list
        .iter()
        .find(|cmd| matches!(cmd, flow_canvas::render::DrawCommand::Rect { .. }));

    match rect_cmd {
        Some(flow_canvas::render::DrawCommand::Rect { pos, size, .. }) => {
            assert_eq!(*pos, Vec2::new(100.0, 100.0));
            assert_eq!(*size, Vec2::new(100.0, 50.0));
        }
        _ => panic!("Expected Rect command not found in draw_list"),
    }
}
