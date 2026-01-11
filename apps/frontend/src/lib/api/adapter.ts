
export type NodeId = number;
export type PortId = number;
export type WireStyle = "Cubic" | "Linear" | "Orthogonal";

export interface PlaygroundNodeData {
    name: string;
    template_id: string;
    settings: Record<string, any>;
}

export interface SerializableNode {
    id: NodeId;
    uuid: string;
    position: [number, number];
    size: [number, number];
    inputs: PortId[];
    outputs: PortId[];
    data: PlaygroundNodeData;
}

export interface SerializableEdge {
    id: string;
    from: PortId;
    to: PortId;
    style: WireStyle;
    path: [number, number][];
    bezier_control_points?: [[number, number], [number, number]];
}

export interface SerializableGraph {
    nodes: Record<NodeId, SerializableNode>;
    edges: Record<string, SerializableEdge>;
    draw_order: NodeId[];
}

export interface NodeTemplate {
    id: string;
    name: string;
    category: string;
    type: string;
    description?: string;
    version?: string;
    platform?: string;
    inputs: any[]; // PortMetadata
    outputs: any[];
    settings: any[]; // SettingMetadata
}

export interface BackendAdapter {
    initSdk(): Promise<void>;
    getNodeTemplates(): Promise<NodeTemplate[]>;
    getGraph(): Promise<SerializableGraph>;
    addNode(templateId: string, x: number, y: number): Promise<string>;
    addEdge(from: PortId, to: PortId): Promise<string>;
    updateNodePosition(id: NodeId, x: number, y: number, commit: boolean): Promise<void>;

    deleteItems(nodes: NodeId[], edges: string[]): Promise<void>;
    copyItems(nodes: NodeId[]): Promise<string>;
    pasteItems(json: string, x: number, y: number): Promise<void>;

    updateNodeSettings(nodeId: NodeId, settings: Record<string, any>): Promise<void>;
    setConnectionWireStyle(id: string, style: WireStyle): Promise<void>;

    undo(): Promise<void>;
    redo(): Promise<void>;

    deploy(): Promise<void>;
    simulateNode(nodeId: NodeId, payload: any, mocks: Record<string, any>): Promise<any>;
}
