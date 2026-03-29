import { getBackend } from "$lib/context/backend.svelte";
import type { GraphState, NodeMetadata, Vec2 } from "$lib/types";

export class CanvasState {
    // State
    graph = $state<GraphState>({
        nodes: {},
        ports: {},
        connections: [],
        draw_order: [],
    });

    templates = $state<Record<string, NodeMetadata>>({});

    // Transform
    offset = $state<Vec2>({ x: 0, y: 0 });
    scale = $state(1);

    // Interaction
    selectedNodes = $state<Set<string>>(new Set());
    selectedEdges = $state<Set<string>>(new Set());

    dragEdgeStart = $state<string | null>(null);
    dragEdgeCurrent = $state<Vec2 | null>(null);
    draggingNode = $state<string | null>(null);
    isDraggingOver = $state(false);

    selectionStart = $state<Vec2 | null>(null);
    selectionCurrent = $state<Vec2 | null>(null);

    mouseWorldPos = $state<Vec2>({ x: 0, y: 0 });

    backend = getBackend();
    
    clipboard = $state<any[]>([]);

    // Workflow metadata
    currentWorkflowId = $state<string | null>(null);
    currentWorkflowName = $state<string>("Untitled Workflow");

    constructor() {
        // any init
    }

    screenToWorld(x: number, y: number): Vec2 {
        return {
            x: (x - this.offset.x) / this.scale,
            y: (y - this.offset.y) / this.scale,
        };
    }

    getPortPosition(portId: string): Vec2 {
        const port = this.graph.ports[portId];
        if (!port) return { x: 0, y: 0 };
        const node = this.graph.nodes[port.node_id];
        if (!node) return { x: 0, y: 0 };

        let index = node.inputs.indexOf(portId);
        let isInput = index !== -1;
        if (!isInput) {
            index = node.outputs.indexOf(portId);
        }
        if (index === -1) return { x: 0, y: 0 };

        const headerHeight = 33;
        const bodyPadding = 8;
        const portHeight = 20;
        const portGap = 4;
        const portStride = portHeight + portGap;

        let yOffset =
            headerHeight + bodyPadding + index * portStride + portHeight / 2;

        if (!isInput) {
            const inputCount = node.inputs.length;
            const inputBlockHeight =
                inputCount > 0 ? inputCount * portStride + 4 : 0;
            yOffset += inputBlockHeight;
        }

        const y = node.position.y + yOffset;
        const x = isInput
            ? node.position.x
            : node.position.x + node.size.x;
        return { x, y };
    }

    async refreshGraph() {
        const [g_json, t] = await Promise.all([
            this.backend.getGraph(),
            this.backend.getTemplates(),
        ]);
        const g = (
            typeof g_json === "string" ? JSON.parse(g_json) : g_json
        ) as GraphState;
        this.graph = g;
        this.templates = t.reduce((acc: any, curr: any) => ({ ...acc, [curr.id]: curr }), {});
    }

    handleSelection(x1: number, y1: number, x2: number, y2: number) {
        // Check intersection (simple AABB)
        const newSelection = new Set(this.selectedNodes);

        const minX = Math.min(x1, x2);
        const minY = Math.min(y1, y2);
        const maxX = Math.max(x1, x2);
        const maxY = Math.max(y1, y2);

        for (const node of Object.values(this.graph.nodes)) {
            const nx = node.position.x;
            const ny = node.position.y;
            const nw = node.size?.x || 200;
            const nh = node.size?.y || 100;

            if (nx < maxX && nx + nw > minX && ny < maxY && ny + nh > minY) {
                newSelection.add(node.id);
            }
        }
        this.selectedNodes = newSelection;

        // ... selection logic ... 
    }

    getObstacles(): any[] {
        return Object.values(this.graph.nodes).map(n => ({
            x: n.position.x,
            y: n.position.y,
            w: n.size?.x || 200,
            h: n.size?.y || 100 // Estimate
        }));
    }

    async copy() {
        this.clipboard = Array.from(this.selectedNodes)
            .map(id => this.graph.nodes[id])
            .filter(Boolean);
    }

    async paste() {
        if (!this.clipboard.length) return;
        
        let newSelection = new Set<string>();
        
        // Find center of clipboard items to paste at mouse position, or just offset 40,40
        await Promise.all(this.clipboard.map(async (node) => {
            // Backend currently only takes template ID and position
            const newId = await this.backend.addNode(
                node.data, 
                node.position.x + 40, 
                node.position.y + 40
            );
            
            // Re-apply configs
            if (node.config) {
                const configPromises = Object.entries(node.config).map(([k, v]) =>
                    this.backend.updateNodeConfig(newId, k, v)
                );
                await Promise.all(configPromises);
            }
            newSelection.add(newId);
        }));
        
        await this.refreshGraph();
        this.selectedNodes = newSelection;
        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
    }

    async duplicate() {
        await this.copy();
        await this.paste();
    }

    async loadFromJson(data: any, id: string) {
        // Clear current graph and rebuild
        await this.backend.newWorkflow();
        
        let newSelection = new Set<string>();
        
        // Ensure nodes is an array or handle object map
        const nodesList = data.nodes ? Object.values(data.nodes) : [];
        const uuidMap = new Map<string, string>(); // oldId -> newId

        await Promise.all((nodesList as any[]).map(async (n) => {
            const newId = await this.backend.addNode(n.data, n.position.x, n.position.y);
            uuidMap.set(n.id, newId);
            
            if (n.config) {
                const configPromises = Object.entries(n.config).map(([k, v]) =>
                    this.backend.updateNodeConfig(newId, k, v)
                );
                await Promise.all(configPromises);
            }
        }));

        // Reconnect edges
        const connections = data.connections || [];
        for (const c of connections as any[]) {
            // Because the frontend API `addNode` recreates ports with specific UUIDs,
            // we'd need a robust programmatic link or just push the loaded JSON to backend directly.
            // For Phase 6 prototype, we rely on the backend doing load internally.
            // But since the current Tauri backend returns empty JSON on load,
            // this loop is a placeholder until FlowCanvas state serde is fully rebuilt.
        }

        this.currentWorkflowId = id;
        this.currentWorkflowName = id; // use ID as name temporarily
        this.selectedNodes = new Set();
        this.selectedEdges = new Set();
        
        await this.refreshGraph();
        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
    }
}
