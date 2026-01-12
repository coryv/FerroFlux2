export interface IBackend {
    init(): Promise<void>;
    getGraph(): Promise<any>; // Replace 'any' with GraphState type later
    getTemplates(): Promise<any[]>; // Replace 'any' with NodeTemplate type
    addNode(templateId: string, x: number, y: number): Promise<string>; // Returns NodeId
}
