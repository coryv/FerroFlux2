import type { IBackend } from '../backend';

export class WebAdapter implements IBackend {
    private mockConfigs = new Map<string, any>();

    async init(): Promise<void> {
        console.log('Web Adapter Initialized (Mock Mode)');
    }

    async getGraph(): Promise<any> {
        return { nodes: {}, ports: {}, connections: [], draw_order: [] };
    }

    async getTemplates(): Promise<any[]> {
        // Mock templates for web demo
        return [
            { id: 'core/trigger', name: 'Manual Trigger', category: 'Triggers' },
            { id: 'core/log', name: 'Log Output', category: 'Utilities' }
        ];
    }

    async addNode(templateId: string, x: number, y: number): Promise<string> {
        console.log(`[Web] Adding node ${templateId} at ${x},${y}`);
        return 'mock-node-id-' + Date.now();
    }

    async connectPorts(from: string, to: string): Promise<string> {
        console.log(`[Web] Connecting ${from} to ${to}`);
        return 'mock-conn-id-' + Date.now();
    }

    async deleteNode(id: string): Promise<string> {
        console.log("WebAdapter: deleteNode", id);
        this.mockConfigs.delete(id); // Clean up mock config
        return "ok";
    }

    async updateNodePosition(id: string, x: number, y: number): Promise<string> {
        console.log("WebAdapter: updateNodePosition", id, x, y);
        return "ok";
    }

    async deleteConnection(id: string): Promise<string> {
        console.log('[WebAdapter] deleteConnection', id);
        return "ok";
    }
    
    async updateNodeConfig(nodeId: string, key: string, value: any): Promise<void> {
        console.log('[WebAdapter] updateNodeConfig', nodeId, key, value);
        let config = this.mockConfigs.get(nodeId) || {};
        config[key] = value;
        this.mockConfigs.set(nodeId, config);
    }
    
    async getNodeConfig(nodeId: string): Promise<Record<string, any>> {
        const configs = this.mockConfigs.get(nodeId);
        return configs || {};
    }

    async executeWorkflow(): Promise<string> {
        console.log("Mock executeWorkflow");
        return "mock_trace_id";
    }

    async simulateNode(nodeId: string, input: any): Promise<string> {
        console.log("Mock simulateNode", nodeId, input);
        return "mock_trace_id";
    }

    async stopExecution(traceId: string): Promise<void> {
        console.log("Mock stopExecution", traceId);
    }

    async onEvent(callback: (event: any) => void): Promise<() => void> {
        // Mock does not emit events
        return () => {};
    }

    async saveWorkflow(name: string): Promise<string> {
        console.log("Mock saveWorkflow", name);
        return "Saved";
    }

    async loadWorkflow(id: string): Promise<string> {
        console.log("Mock loadWorkflow", id);
        return "{}";
    }

    async listWorkflows(): Promise<Array<{ id: string, name: string, last_modified: number, node_count: number }>> {
        return [{ id: "mock_1", name: "Mock Workflow", last_modified: Date.now(), node_count: 5 }];
    }

    async newWorkflow(): Promise<void> {
        console.log("Mock newWorkflow");
    }
}
