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
}
