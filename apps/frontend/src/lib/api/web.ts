
import type { BackendAdapter, NodeId, PortId, WireStyle, NodeTemplate, SerializableGraph } from './adapter';

export class WebAdapter implements BackendAdapter {
    async initSdk(): Promise<void> {
        console.log('[WebAdapter] SDK Initialized');
    }

    async getNodeTemplates(): Promise<NodeTemplate[]> {
        return [
            {
                id: 'core.log',
                name: 'Log',
                category: 'Utility',
                type: 'Action',
                inputs: [{ name: 'in', type: 'any' }],
                outputs: [{ name: 'out', type: 'any' }],
                settings: []
            },
            {
                id: 'core.trigger',
                name: 'Webhook',
                category: 'Trigger',
                type: 'Trigger',
                inputs: [],
                outputs: [{ name: 'trigger', type: 'json' }],
                settings: [{ name: 'port', label: 'Port', type: 'number', default: 8080 }]
            }
        ];
    }

    async getGraph(): Promise<SerializableGraph> {
        return {
            nodes: {},
            edges: {},
            draw_order: []
        };
    }

    async addNode(templateId: string, x: number, y: number): Promise<string> {
        console.log('[WebAdapter] addNode', templateId, x, y);
        return 'mock-node-id';
    }

    async addEdge(from: PortId, to: PortId): Promise<string> {
        console.log('[WebAdapter] addEdge', from, to);
        return 'mock-edge-id';
    }

    async updateNodePosition(id: NodeId, x: number, y: number, commit: boolean): Promise<void> {
        // console.log('[WebAdapter] updatePos', id, x, y, commit);
    }

    async deleteItems(nodes: NodeId[], edges: string[]): Promise<void> {
        console.log('[WebAdapter] deleteItems', nodes, edges);
    }

    async copyItems(nodes: NodeId[]): Promise<string> {
        return '{}';
    }

    async pasteItems(json: string, x: number, y: number): Promise<void> {
        console.log('[WebAdapter] pasteItems');
    }

    async updateNodeSettings(nodeId: NodeId, settings: Record<string, any>): Promise<void> {
        console.log('[WebAdapter] updateSettings', nodeId, settings);
    }

    async setConnectionWireStyle(id: string, style: WireStyle): Promise<void> {
        console.log('[WebAdapter] setWireStyle', id, style);
    }

    async undo(): Promise<void> {
        console.log('[WebAdapter] undo');
    }

    async redo(): Promise<void> {
        console.log('[WebAdapter] redo');
    }

    async deploy(): Promise<void> {
        console.log('[WebAdapter] deploy');
    }

    async simulateNode(nodeId: NodeId, payload: any, mocks: Record<string, any>): Promise<any> {
        console.log('[WebAdapter] simulateNode');
        return { simulated: true };
    }
}
