<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { GraphState, SerializableNode } from "$lib/types";
    import Node from "./Node.svelte";
    import Edge from "./Edge.svelte";
    import { findPortPosition, type Obstacle } from "../utils/routing";

    let { graph, onRefresh } = $props<{
        graph: GraphState;
        onRefresh: () => Promise<void>;
    }>();

    let pan = $state({ x: 0, y: 0 });
    let zoom = $state(1);
    let isDraggingCanvas = $state(false);
    let draggingNodeId = $state<string | null>(null);
    let selectedNodeId = $state<string | null>(null);
    let canvasElement = $state<HTMLDivElement | null>(null);

    // Connection state
    let connectingFrom = $state<{
        nodeId: string;
        portId: string;
        isOutput: boolean;
    } | null>(null);
    let tempEdgeEnd = $state<{ x: number; y: number } | null>(null);

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const zoomSpeed = 0.001;
        const newZoom = Math.max(0.1, Math.min(5, zoom - e.deltaY * zoomSpeed));
        zoom = newZoom;
    }

    function onMouseDown(e: MouseEvent) {
        if (e.button === 0 || e.button === 1) {
            e.preventDefault();
            isDraggingCanvas = true;
            selectedNodeId = null;
        }
    }

    async function onNodeMouseDown(e: MouseEvent, nodeId: string) {
        e.stopPropagation();
        e.preventDefault();
        draggingNodeId = nodeId;
        selectedNodeId = nodeId;

        // Bring to front in backend
        try {
            await invoke("bring_to_front", { id: nodeId });
            // Optimistic update
            graph.draw_order = [
                ...graph.draw_order.filter((id: string) => id !== nodeId),
                nodeId,
            ];
        } catch (err) {
            console.error("Failed to bring node to front:", err);
        }
    }

    function onPortMouseDown(
        e: MouseEvent,
        nodeId: string,
        portId: string,
        isOutput: boolean,
    ) {
        e.stopPropagation();
        e.preventDefault();
        connectingFrom = { nodeId, portId, isOutput };
        tempEdgeEnd = screenToWorld(e.clientX, e.clientY);
    }

    async function onPortMouseUp(
        e: MouseEvent,
        nodeId: string,
        portId: string,
        isOutput: boolean,
    ) {
        if (!connectingFrom) return;
        e.stopPropagation();

        // Valid connection rule: from output to input (or vice versa) and different ports
        if (
            connectingFrom.isOutput !== isOutput &&
            connectingFrom.portId !== portId
        ) {
            try {
                const from = connectingFrom.isOutput
                    ? connectingFrom.portId
                    : portId;
                const to = connectingFrom.isOutput
                    ? portId
                    : connectingFrom.portId;

                await invoke("add_edge", { from, to });
                await onRefresh();
            } catch (err) {
                console.error("Failed to connect ports:", err);
            }
        }

        connectingFrom = null;
        tempEdgeEnd = null;
    }

    function onMouseMove(e: MouseEvent) {
        if (draggingNodeId && graph.nodes[draggingNodeId]) {
            const dx = e.movementX / zoom;
            const dy = e.movementY / zoom;

            const node = graph.nodes[draggingNodeId];
            node.position[0] += dx;
            node.position[1] += dy;
        } else if (connectingFrom) {
            tempEdgeEnd = screenToWorld(e.clientX, e.clientY);
        } else if (isDraggingCanvas) {
            pan.x += e.movementX;
            pan.y += e.movementY;
        }
    }

    function screenToWorld(sx: number, sy: number) {
        if (!canvasElement) return { x: 0, y: 0 };
        const rect = canvasElement.getBoundingClientRect();
        return {
            x: (sx - rect.left - pan.x) / zoom,
            y: (sy - rect.top - pan.y) / zoom,
        };
    }

    async function onMouseUp() {
        if (draggingNodeId && graph.nodes[draggingNodeId]) {
            const node = graph.nodes[draggingNodeId];
            try {
                await invoke("update_node_position", {
                    id: draggingNodeId,
                    x: node.position[0],
                    y: node.position[1],
                });
            } catch (e) {
                console.error("Failed to sync node position:", e);
            }
        }
        isDraggingCanvas = false;
        draggingNodeId = null;
        connectingFrom = null;
        tempEdgeEnd = null;
    }

    function getNodes(g: GraphState): SerializableNode[] {
        if (!g || !g.nodes) return [];
        // Use draw_order if available, otherwise fallback to object values
        if (g.draw_order && g.draw_order.length > 0) {
            return g.draw_order
                .map((id) => g.nodes[id])
                .filter((node) => node !== undefined);
        }
        return Object.values(g.nodes);
    }

    function getEdges(g: GraphState) {
        if (!g || !g.edges) return [];
        return Object.values(g.edges);
    }

    let selectedNodePorts = $derived(
        selectedNodeId && graph.nodes[selectedNodeId]
            ? new Set([
                  ...graph.nodes[selectedNodeId].inputs,
                  ...graph.nodes[selectedNodeId].outputs,
              ])
            : new Set<string>(),
    );

    function getSortedEdges(g: GraphState) {
        if (!g || !g.edges) return [];
        const edges = Object.values(g.edges);
        if (!selectedNodeId || !graph.nodes[selectedNodeId]) return edges;

        // Collect all ports belonging to the selected node
        const node = graph.nodes[selectedNodeId];
        const connectedPorts = new Set([...node.inputs, ...node.outputs]);

        return [...edges].sort((a, b) => {
            const aConn =
                connectedPorts.has(a.from) || connectedPorts.has(a.to);
            const bConn =
                connectedPorts.has(b.from) || connectedPorts.has(b.to);
            if (aConn && !bConn) return 1;
            if (!aConn && bConn) return -1;
            return 0;
        });
    }

    function getObstacles(g: GraphState): Obstacle[] {
        return Object.values(g.nodes).map((n) => ({
            min: { x: n.position[0], y: n.position[1] },
            max: {
                x: n.position[0] + n.size[0],
                y: n.position[1] + n.size[1],
            },
        }));
    }

    async function cycleEdgeStyle(edgeId: string, currentStyle: string) {
        const styles: string[] = ["Cubic", "Linear", "Orthogonal"];
        const nextStyle =
            styles[(styles.indexOf(currentStyle) + 1) % styles.length];

        try {
            await invoke("set_connection_wire_style", {
                id: edgeId,
                style: nextStyle,
            });
            await onRefresh();
        } catch (err) {
            console.error("Failed to cycle edge style:", err);
        }
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    bind:this={canvasElement}
    class="canvas"
    onwheel={onWheel}
    onmousedown={onMouseDown}
    onmousemove={onMouseMove}
    onmouseup={onMouseUp}
    onmouseleave={onMouseUp}
    style="background-position: {pan.x}px {pan.y}px; background-size: {20 *
        zoom}px {20 * zoom}px;"
>
    <div
        class="transform-layer"
        style="transform: translate({pan.x}px, {pan.y}px) scale({zoom}); transform-origin: 0 0;"
    >
        <!-- Edges SVG Layer -->
        <svg class="edges-svg">
            <defs>
                <marker
                    id="arrowhead"
                    markerWidth="10"
                    markerHeight="7"
                    refX="0"
                    refY="3.5"
                    orient="auto"
                >
                    <polygon points="0 0, 10 3.5, 0 7" fill="#666" />
                </marker>
            </defs>
            {#if graph}
                {@const nodes = getNodes(graph)}
                {@const obstacles = getObstacles(graph)}
                {#each getSortedEdges(graph) as edge (edge.id)}
                    {@const start = findPortPosition(nodes, edge.from)}
                    {@const end = findPortPosition(nodes, edge.to)}
                    {#if start && end}
                        <Edge
                            {start}
                            {end}
                            style={edge.style}
                            {obstacles}
                            selected={selectedNodePorts.has(edge.from) ||
                                selectedNodePorts.has(edge.to)}
                            onclick={() => cycleEdgeStyle(edge.id, edge.style)}
                        />
                    {/if}
                {/each}
            {/if}

            <!-- Temporary Edge -->
            {#if connectingFrom && tempEdgeEnd}
                {@const nodes = getNodes(graph)}
                {@const start = findPortPosition(nodes, connectingFrom.portId)}
                {#if start}
                    <Edge
                        {start}
                        end={tempEdgeEnd}
                        style="Cubic"
                        selected={true}
                    />
                {/if}
            {/if}
        </svg>

        {#if graph}
            {#each getNodes(graph) as node (node.id)}
                <Node
                    {node}
                    onMouseDown={onNodeMouseDown}
                    {onPortMouseDown}
                    {onPortMouseUp}
                />
            {/each}
        {/if}
    </div>
</div>

<style>
    .canvas {
        position: relative;
        width: 100vw;
        height: calc(100vh - 40px);
        background: #111;
        overflow: hidden;
        cursor: grab;
        background-image: radial-gradient(#333 1px, transparent 1px);
    }
    .canvas:active {
        cursor: grabbing;
    }
    .transform-layer {
        width: 100%;
        height: 100%;
        position: absolute;
        top: 0;
        left: 0;
    }
    .edges-svg {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
        overflow: visible;
    }
</style>
