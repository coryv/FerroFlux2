import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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

    async executeWorkflow(): Promise<string> {
        return await invoke("execute_workflow");
    }

    async simulateNode(nodeId: string, input: any): Promise<string> {
        return await invoke("simulate_node", { nodeId, input });
    }

    async stopExecution(traceId: string): Promise<void> {
        return await invoke("stop_execution", { traceId });
    }

    async onEvent(callback: (event: any) => void): Promise<() => void> {
        const unlisten = await listen('system-event', (event) => {
            callback(event.payload);
        });
        return unlisten;
    }

    async saveWorkflow(name: string): Promise<string> {
        return await invoke("save_workflow", { name });
    }

    async loadWorkflow(id: string): Promise<string> {
        return await invoke("load_workflow", { id });
    }

    async listWorkflows(): Promise<Array<{ id: string, name: string, last_modified: number, node_count: number }>> {
        return await invoke("list_workflows");
    }

    async newWorkflow(): Promise<void> {
        return await invoke("new_workflow");
    }
}
