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
    executeWorkflow(): Promise<string>;
    simulateNode(nodeId: string, input: any): Promise<string>;
    stopExecution(traceId: string): Promise<void>;
    onEvent(callback: (event: any) => void): Promise<() => void>;
    saveWorkflow(name: string): Promise<string>;
    loadWorkflow(id: string): Promise<string>; // Returns graph state JSON string
    listWorkflows(): Promise<Array<{ id: string, name: string, last_modified: number, node_count: number }>>;
    newWorkflow(): Promise<void>;
}
