// Types matching Rust SerializableGraph
export interface SerializableNode {
    id: string;
    uuid: string;
    position: [number, number];
    size: [number, number];
    data: { name: string; node_type?: string };
}

export interface GraphState {
    nodes: Record<string, SerializableNode>;
}
