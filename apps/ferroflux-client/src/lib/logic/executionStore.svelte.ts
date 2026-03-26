import type { IBackend } from '../api/backend';

export type NodeStatus = 'idle' | 'running' | 'success' | 'error';
export type EdgeStatus = 'idle' | 'running';

export interface NodeExecutionState {
    status: NodeStatus;
    executionMs: number;
    error?: string;
    details?: any;
}

export interface SystemEvent {
    type: string;
    data: any;
}

export class ExecutionStore {
    // Current trace execution state
    nodeStates = $state<Record<string, NodeExecutionState>>({});
    edgeStates = $state<Record<string, EdgeStatus>>({});
    logs = $state<Array<{ level: string, message: string, trace_id: string, timestamp: number }>>([]);
    
    currentTraceId = $state<string | null>(null);
    isRunning = $state(false);
    unlistenFn: (() => void) | null = null;
    backend: IBackend | null = null;

    constructor() {}

    async init(backend: IBackend) {
        this.backend = backend;
        this.unlistenFn = await backend.onEvent(this.handleEvent.bind(this));
    }

    destroy() {
        if (this.unlistenFn) {
            this.unlistenFn();
            this.unlistenFn = null;
        }
    }

    async runWorkflow() {
        if (!this.backend) return;
        this.reset();
        this.isRunning = true;
        try {
            this.currentTraceId = await this.backend.executeWorkflow();
        } catch (e) {
            console.error("Failed to run workflow", e);
            this.isRunning = false;
        }
    }

    async stop() {
        if (!this.backend || !this.currentTraceId) {
            this.isRunning = false;
            return;
        }
        await this.backend.stopExecution(this.currentTraceId);
        this.isRunning = false;
    }

    reset() {
        this.nodeStates = {};
        this.edgeStates = {};
        this.logs = [];
        this.currentTraceId = null;
        this.isRunning = false;
    }

    private handleEvent(event: SystemEvent) {
        if (!this.isRunning) return;

        switch (event.type) {
            case 'Log':
                this.logs.push(event.data);
                break;
            case 'NodeTelemetry':
                this.nodeStates[event.data.node_id] = {
                    status: event.data.success ? 'success' : 'error',
                    executionMs: event.data.execution_ms,
                    details: event.data.details
                };
                break;
            case 'NodeError':
                console.error("Node error:", event.data);
                this.nodeStates[event.data.node_id] = {
                    status: 'error',
                    executionMs: 0,
                    error: event.data.error
                };
                break;
            case 'WorkflowUpdate':
                if (event.data.status === 'Completed' || event.data.status === 'Failed') {
                    this.isRunning = false;
                }
                break;
            case 'EdgeTraversal':
                // Temporarily flash edge logic could go here
                break;
            case 'AgentActivity':
                // Update specific node state with partial progress
                const state = this.nodeStates[event.data.node_id];
                if (!state || state.status === 'idle') {
                    this.nodeStates[event.data.node_id] = { status: 'running', executionMs: 0 };
                }
                break;
        }
    }
}

export const executionStore = new ExecutionStore();
