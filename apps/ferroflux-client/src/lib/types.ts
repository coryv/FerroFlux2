export interface PortMetadata {
    name: string;
    data_type: string;
}

export interface NodeMetadata {
    id: string;
    name: string;
    category: string;
    node_type?: string;
    description?: string;
    inputs: PortMetadata[];
    outputs: PortMetadata[];
    settings?: any[];
    type?: string;
    meta?: any;
    interface?: any;
}

// Runtime Types (Mirroring flow_canvas/ferroflux_core)
export interface Vec2 {
    x: number;
    y: number;
}

export interface Port {
    id: string; // PortId
    node_id: string; // NodeId
    name: string;
    data_type: string;
    kind: 'Input' | 'Output';
}

export interface NodeData {
    id: string; // SlotMap Key (NodeId)
    uuid: string; // Logical UUID
    position: Vec2;
    size: Vec2;
    inputs: string[]; // List of PortIds
    outputs: string[]; // List of PortIds
    data: string; // Template ID (e.g. "core.action.log")
    config?: Record<string, any>;
    // styles, flags, etc.
}

export interface GraphState {
    nodes: Record<string, NodeData>;
    ports: Record<string, Port>;
    connections: any[]; // Now an array of ConnectionDto
    draw_order: string[];
}
