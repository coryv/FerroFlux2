import type { BackendAdapter, NodeId, PortId, WireStyle, NodeTemplate, SerializableGraph } from './adapter';
import { invoke } from '@tauri-apps/api/core';

export class TauriAdapter implements BackendAdapter {
    async initSdk(): Promise<void> {
        return invoke('init_sdk');
    }

    async getNodeTemplates(): Promise<NodeTemplate[]> {
        return invoke('get_node_templates');
    }

    async getGraph(): Promise<SerializableGraph> {
        return invoke('get_graph');
    }

    async addNode(templateId: string, x: number, y: number): Promise<string> {
        return invoke('add_node', { templateId, x, y });
    }

    async addEdge(from: PortId, to: PortId): Promise<string> {
        return invoke('add_edge', { from, to });
    }

    async updateNodePosition(id: NodeId, x: number, y: number, commit: boolean): Promise<void> {
        return invoke('update_node_position', { id, x, y, commit });
    }

    async deleteItems(nodes: NodeId[], edges: string[]): Promise<void> {
        return invoke('delete_items', { nodes, edges });
    }

    async copyItems(nodes: NodeId[]): Promise<string> {
        return invoke('copy_items', { nodes });
    }

    async pasteItems(json: string, x: number, y: number): Promise<void> {
        return invoke('paste_items', { json, x, y });
    }

    async updateNodeSettings(nodeId: NodeId, settings: Record<string, any>): Promise<void> {
        return invoke('update_node_settings', { nodeId, settings });
    }

    async setConnectionWireStyle(id: string, style: WireStyle): Promise<void> {
        return invoke('set_connection_wire_style', { id, style });
    }

    async undo(): Promise<void> {
        return invoke('undo');
    }

    async redo(): Promise<void> {
        return invoke('redo');
    }

    async deploy(): Promise<void> {
        return invoke('deploy');
    }

    async simulateNode(nodeId: NodeId, payload: any, mocks: Record<string, any>): Promise<any> {
        return invoke('simulate_node', { nodeId, payload, mocks });
    }
}
