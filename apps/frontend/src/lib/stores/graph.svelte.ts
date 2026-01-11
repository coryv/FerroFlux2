
import { getContext, setContext } from 'svelte';
import type { BackendAdapter, SerializableGraph, SerializableNode, SerializableEdge, NodeId, PortId } from '../api/adapter';
import { useSdk } from '../context/sdk.svelte';

const GRAPH_KEY = Symbol('GRAPH');

export class GraphState {
    // Svelte 5 Runes State
    nodes = $state<Record<NodeId, SerializableNode>>({});
    edges = $state<Record<string, SerializableEdge>>({});
    drawOrder = $state<NodeId[]>([]);

    // View State
    scale = $state(1);
    pan = $state({ x: 0, y: 0 });
    selectedNodes = $state<Set<NodeId>>(new Set());

    // Dependencies
    adapter: BackendAdapter;

    constructor() {
        this.adapter = useSdk();
        this.loadGraph();
    }

    async loadGraph() {
        try {
            const graph = await this.adapter.getGraph();
            this.nodes = graph.nodes;
            this.edges = graph.edges;
            this.drawOrder = graph.draw_order;
        } catch (e) {
            console.error('Failed to load graph:', e);
        }
    }

    // --- Actions ---

    async addNode(templateId: string, x: number, y: number) {
        // Optimistic update difficult for creation since we need ID from backend.
        // For now, we wait for backend.
        // TODO: Implement optimistic creation with temp IDs if latency becomes issue.
        try {
            const nodeIdStr = await this.adapter.addNode(templateId, x, y);
            // Reload graph to get the new node data
            // Optimization: Backend could return the full node object
            await this.loadGraph();
        } catch (e) {
            console.error('Failed to add node:', e);
        }
    }

    async updateNodePosition(id: NodeId, x: number, y: number, commit: boolean = false) {
        // Optimistic Update
        const node = this.nodes[id];
        if (node) {
            node.position = [x, y];
        }

        // Sync with backend
        this.adapter.updateNodePosition(id, x, y, commit).catch(e => {
            console.error('Failed to sync position:', e);
            // Revert on failure? require robust sync logic.
            // For now, reload graph on error to ensure consistency
            this.loadGraph();
        });
    }

    async connect(from: PortId, to: PortId) {
        try {
            await this.adapter.addEdge(from, to);
            await this.loadGraph();
        } catch (e) {
            console.error('Failed to connect:', e);
        }
    }

    selectNode(id: NodeId, exclusive: boolean = true) {
        if (exclusive) {
            this.selectedNodes.clear();
        }
        this.selectedNodes.add(id);
        // Force reactivity if needed, Set usually needs help or new Set
        this.selectedNodes = new Set(this.selectedNodes);
    }

    clearSelection() {
        this.selectedNodes = new Set();
    }
}

export function initGraphState() {
    const graph = new GraphState();
    setContext(GRAPH_KEY, graph);
    return graph;
}

export function useGraph() {
    const graph = getContext<GraphState>(GRAPH_KEY);
    if (!graph) throw new Error('Graph Context not found');
    return graph;
}
