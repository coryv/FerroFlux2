import { invoke } from '@tauri-apps/api/core';
import type { IBackend } from '../backend';

export class TauriAdapter implements IBackend {
    async init(): Promise<void> {
        console.log('Tauri Adapter Initialized');
        // Any specific Tauri setup
    }

    async getGraph(): Promise<any> {
        return await invoke('get_graph');
    }

    async getTemplates(): Promise<any[]> {
        return await invoke('get_node_templates');
    }

    async addNode(templateId: string, x: number, y: number): Promise<string> {
        return await invoke('add_node', { templateId, x, y });
    }

    async connectPorts(from: string, to: string): Promise<string> {
        return await invoke('connect_ports', { from, to });
    }

    async deleteConnection(id: string): Promise<string> {
        return await invoke("delete_connection", { id });
    }

    async updateNodePosition(id: string, x: number, y: number): Promise<string> {
        return await invoke("update_node_position", { id, x, y });
    }

    async deleteNode(id: string): Promise<string> {
        return await invoke("delete_node", { id });
    }

    async updateNodeConfig(nodeId: string, key: string, value: any): Promise<void> {
        return await invoke("update_node_config", { nodeId, key, value });
    }

    async getNodeConfig(nodeId: string): Promise<Record<string, any>> {
        return await invoke("get_node_config", { nodeId });
    }
}
