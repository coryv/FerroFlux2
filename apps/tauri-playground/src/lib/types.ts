// Types matching Rust SerializableGraph
export interface SerializableNode {
    id: string;
    uuid: string;
    position: [number, number];
    size: [number, number];
    inputs: string[];
    outputs: string[];
    data: { name: string; node_type?: string };
}

export type WireStyle = "Cubic" | "Linear" | "Orthogonal";

export interface SerializableEdge {
    id: string;
    from: string;
    to: string;
    style: WireStyle;
    path: [number, number][];
    bezier_control_points?: [[number, number], [number, number]];
}

export interface GraphState {
    nodes: Record<string, SerializableNode>;
    edges: Record<string, SerializableEdge>;
    draw_order: string[];
}
