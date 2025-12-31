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

export interface PortTemplate {
    name: string;
    data_type: string;
}

export interface NodeTemplate {
    id: string;
    name: string;
    category: string;
    description?: string;
    inputs: PortTemplate[];
    outputs: PortTemplate[];
    default_width?: number;
    default_height?: number;
}
