use flow_canvas::history::HistoryManager;
use flow_canvas::model::{GraphState, Node, NodeFlags, Uuid};
use glam::Vec2;

#[test]
fn test_history_basic() {
    let mut graph: GraphState<String> = GraphState::default();
    let mut history = HistoryManager::<String>::new(5);

    // Initial State: 1 Node
    let node_id = graph.nodes.insert(Node {
        id: flow_canvas::model::NodeId::default(),
        uuid: Uuid::new_v4(),
        position: Vec2::ZERO,
        size: Vec2::ONE,
        inputs: vec![],
        outputs: vec![],
        data: "Init".to_string(),
        flags: NodeFlags::default(),
        style: None,
    });

    // 1. Commit Initial State
    history.commit(&graph);

    // 2. Modify State (Move Node)
    graph.nodes[node_id].position = Vec2::new(100.0, 100.0);

    // 3. Commit Modified State
    history.commit(&graph);

    // 4. Modify State Again (Delete Node - sim)
    graph.nodes.remove(node_id);
    assert!(graph.nodes.is_empty());

    // --- UNDO ---

    // Undo 1: Should bring back node at (100, 100)
    assert!(history.undo(&mut graph));
    assert!(!graph.nodes.is_empty());
    // Since SlotMap keys are stable in this clone-based snapshot, we can use original ID?
    // Yes, because we cloned the SlotMap.
    assert_eq!(graph.nodes[node_id].position, Vec2::new(100.0, 100.0));

    // Undo 2: Should bring back node at (0, 0)
    assert!(history.undo(&mut graph));
    assert_eq!(graph.nodes[node_id].position, Vec2::ZERO);

    // Undo 3: Should fail (stack empty)
    assert!(!history.undo(&mut graph));

    // --- REDO ---

    // Redo 1: Back to (100, 100)
    assert!(history.redo(&mut graph));
    assert_eq!(graph.nodes[node_id].position, Vec2::new(100.0, 100.0));

    // Redo 2: Back to Empty
    assert!(history.redo(&mut graph));
    assert!(graph.nodes.is_empty());

    // Redo 3: Should fail
    assert!(!history.redo(&mut graph));
}
