export interface IBackend {
    init(): Promise<void>;
    getGraph(): Promise<any>; // Replace 'any' with GraphState type later
    getTemplates(): Promise<any[]>; // Replace 'any' with NodeTemplate type
    addNode(templateId: string, x: number, y: number): Promise<string>; // Returns NodeId
    connectPorts(from: string, to: string): Promise<string>; // Returns ConnectionId
    deleteConnection(id: string): Promise<string>;
    updateNodePosition(id: string, x: number, y: number): Promise<string>;
    deleteNode(id: string): Promise<string>;
    updateNodeConfig(nodeId: string, key: string, value: any): Promise<void>;
    getNodeConfig(nodeId: string): Promise<Record<string, any>>;
}
