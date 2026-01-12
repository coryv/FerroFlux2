import type { IBackend } from '../backend';

export class WebAdapter implements IBackend {
    async init(): Promise<void> {
        console.log('Web Adapter Initialized (Mock Mode)');
    }

    async getGraph(): Promise<any> {
        return { nodes: [], edges: [] };
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
}
