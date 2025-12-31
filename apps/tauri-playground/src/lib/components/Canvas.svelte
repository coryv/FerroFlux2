<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { GraphState, SerializableNode } from "$lib/types";
    import Node from "./Node.svelte";
    import Edge from "./Edge.svelte";
    import Toolbar from "./Toolbar.svelte";
    import { findPortPosition, type Obstacle } from "../utils/routing";
    import { SvelteSet } from "svelte/reactivity";
    import { onMount } from "svelte";

    let { graph, onRefresh, status, onDeploy } = $props<{
        graph: GraphState;
        onRefresh: () => Promise<void>;
        status: string;
        onDeploy: () => void;
    }>();

    onMount(() => {
        invoke("log_js", { msg: "Canvas: Component mounted" });
    });

    let pan = $state({ x: 0, y: 0 });
    let zoom = $state(1);
    let isDraggingCanvas = $state(false);
    let draggingNodeId = $state<string | null>(null);
    let selectedNodeIds = $state(new SvelteSet<string>());
    let canvasElement = $state<HTMLDivElement | null>(null);
    let globalEdgeStyle = $state<"Cubic" | "Linear" | "Orthogonal">("Cubic");
    let lastMousePos = $state({ x: 0, y: 0 }); // World space
    let internalClipboard = $state<string | null>(null);

    // Selection box state
    let isBoxSelecting = $state(false);
    let boxStart = $state({ x: 0, y: 0 }); // Screen space
    let boxEnd = $state({ x: 0, y: 0 }); // Screen space

    // Connection state
    let connectingFrom = $state<{
        nodeId: string;
        portId: string;
        isOutput: boolean;
    } | null>(null);
    let tempEdgeEnd = $state<{ x: number; y: number } | null>(null);

    // Drop targeting
    let isDragOver = $state(false);
    let dragEnterCount = 0;

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const zoomSpeed = 0.001;
        const pinchSpeed = 0.01;
        const isZoom = e.ctrlKey || e.shiftKey;

        if (isZoom) {
            const delta = e.ctrlKey
                ? -e.deltaY * pinchSpeed
                : -e.deltaY * zoomSpeed;
            const factor = Math.pow(1.1, delta);
            const newZoom = Math.max(0.1, Math.min(5, zoom * factor));
            const rect = canvasElement?.getBoundingClientRect();
            if (rect) {
                const mouseX = e.clientX - rect.left;
                const mouseY = e.clientY - rect.top;
                const worldX = (mouseX - pan.x) / zoom;
                const worldY = (mouseY - pan.y) / zoom;
                zoom = newZoom;
                pan.x = mouseX - worldX * zoom;
                pan.y = mouseY - worldY * zoom;
            }
        } else {
            pan.x -= e.deltaX;
            pan.y -= e.deltaY;
        }
    }

    let wasPanned = false;
    function onMouseDown(e: MouseEvent) {
        invoke("log_js", {
            msg: `Canvas: onMouseDown button=${e.button} shift=${e.shiftKey}`,
        });
        if (e.button === 2 || e.button === 1) {
            e.preventDefault();
            isDraggingCanvas = true;
            wasPanned = false;
        } else if (e.button === 0) {
            if (!e.shiftKey) {
                selectedNodeIds.clear();
            }
            isBoxSelecting = true;
            boxStart = { x: e.clientX, y: e.clientY };
            boxEnd = { x: e.clientX, y: e.clientY };
        }
    }

    function onContextMenu(e: MouseEvent) {
        if (wasPanned) {
            e.preventDefault();
        }
    }

    async function onNodeMouseDown(e: MouseEvent, nodeId: string) {
        e.stopPropagation();
        e.preventDefault();
        draggingNodeId = nodeId;

        if (e.shiftKey) {
            if (selectedNodeIds.has(nodeId)) {
                selectedNodeIds.delete(nodeId);
            } else {
                selectedNodeIds.add(nodeId);
            }
        } else {
            if (!selectedNodeIds.has(nodeId)) {
                selectedNodeIds.clear();
                selectedNodeIds.add(nodeId);
            }
        }

        try {
            await invoke("bring_to_front", { id: nodeId });
            graph.draw_order = [
                ...graph.draw_order.filter((id: string) => id !== nodeId),
                nodeId,
            ];
        } catch (err) {
            console.error(err);
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
                console.error(err);
            }
        }
        connectingFrom = null;
        tempEdgeEnd = null;
    }

    function onMouseMove(e: MouseEvent) {
        // if (isBoxSelecting || draggingNodeId || isDraggingCanvas) {
        //     // Only log if something is actually happening periodically
        //     // invoke("log_js", { msg: "Canvas: onMouseMove" });
        // }
        lastMousePos = screenToWorld(e.clientX, e.clientY);
        if (draggingNodeId && graph.nodes[draggingNodeId]) {
            const dx = e.movementX / zoom;
            const dy = e.movementY / zoom;
            const node = graph.nodes[draggingNodeId];
            node.position[0] += dx;
            node.position[1] += dy;
        } else if (isBoxSelecting) {
            boxEnd = { x: e.clientX, y: e.clientY };
        } else if (isDraggingCanvas) {
            pan.x += e.movementX;
            pan.y += e.movementY;
            if (Math.abs(e.movementX) > 2 || Math.abs(e.movementY) > 2) {
                wasPanned = true;
            }
        } else if (connectingFrom) {
            tempEdgeEnd = screenToWorld(e.clientX, e.clientY);
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
                    commit: true,
                });
            } catch (e) {
                console.error(e);
            }
        }

        if (isBoxSelecting) {
            const s = screenToWorld(boxStart.x, boxStart.y);
            const e = screenToWorld(boxEnd.x, boxEnd.y);
            const minX = Math.min(s.x, e.x);
            const maxX = Math.max(s.x, e.x);
            const minY = Math.min(s.y, e.y);
            const maxY = Math.max(s.y, e.y);

            for (const node of Object.values(
                graph.nodes,
            ) as SerializableNode[]) {
                const nx = node.position[0];
                const ny = node.position[1];
                const nw = node.size[0];
                const nh = node.size[1];
                if (
                    nx < maxX &&
                    nx + nw > minX &&
                    ny < maxY &&
                    ny + nh > minY
                ) {
                    selectedNodeIds.add(node.id);
                }
            }
        }

        isDraggingCanvas = false;
        draggingNodeId = null;
        connectingFrom = null;
        tempEdgeEnd = null;
        isBoxSelecting = false;
    }

    async function onUndo() {
        try {
            await invoke("undo");
            await onRefresh();
        } catch (e) {
            console.warn(e);
        }
    }

    async function onRedo() {
        try {
            await invoke("redo");
            await onRefresh();
        } catch (e) {
            console.warn(e);
        }
    }

    async function setGlobalStyle(style: "Cubic" | "Linear" | "Orthogonal") {
        try {
            globalEdgeStyle = style;
            await invoke("set_all_connection_wire_styles", { style });
            await onRefresh();
        } catch (e) {
            console.error(e);
        }
    }

    async function onDelete() {
        if (selectedNodeIds.size === 0) return;
        try {
            await invoke("delete_items", {
                nodes: Array.from(selectedNodeIds),
                edges: [],
            });
            selectedNodeIds.clear();
            await onRefresh();
        } catch (e) {
            console.error(e);
        }
    }

    function onKeyDown(e: KeyboardEvent) {
        const isMod = e.metaKey || e.ctrlKey;
        if (isMod && e.key.toLowerCase() === "z") {
            e.preventDefault();
            if (e.shiftKey) onRedo();
            else onUndo();
        } else if (isMod && e.key.toLowerCase() === "c") {
            onCopy();
        } else if (isMod && e.key.toLowerCase() === "v") {
            onPaste();
        } else if (e.key === "Delete" || e.key === "Backspace") {
            onDelete();
        }
    }

    async function onCopy() {
        if (selectedNodeIds.size === 0) return;
        try {
            internalClipboard = await invoke("copy_items", {
                nodes: Array.from(selectedNodeIds),
            });
        } catch (e) {
            console.error(e);
        }
    }

    async function onPaste() {
        if (!internalClipboard) return;
        try {
            await invoke("paste_items", {
                json: internalClipboard,
                x: lastMousePos.x,
                y: lastMousePos.y,
            });
            await onRefresh();
        } catch (e) {
            console.error(e);
        }
    }

    function getNodes(g: GraphState): SerializableNode[] {
        if (!g || !g.nodes) return [];
        if (g.draw_order && g.draw_order.length > 0) {
            return g.draw_order
                .map((id) => g.nodes[id])
                .filter((n) => n !== undefined);
        }
        return Object.values(g.nodes);
    }

    let selectedNodePorts = $derived.by(() => {
        const ports = new Set<string>();
        for (const id of selectedNodeIds) {
            const node = graph.nodes[id];
            if (node) {
                node.inputs.forEach((p: string) => ports.add(p));
                node.outputs.forEach((p: string) => ports.add(p));
            }
        }
        return ports;
    });

    function getSortedEdges(g: GraphState) {
        if (!g || !g.edges) return [];
        const edges = Object.values(g.edges);
        if (selectedNodeIds.size === 0) return edges;
        return [...edges].sort((a, b) => {
            const aConn =
                selectedNodePorts.has(a.from) || selectedNodePorts.has(a.to);
            const bConn =
                selectedNodePorts.has(b.from) || selectedNodePorts.has(b.to);
            if (aConn && !bConn) return 1;
            if (!aConn && bConn) return -1;
            return 0;
        });
    }

    function getObstacles(g: GraphState): Obstacle[] {
        return Object.values(g.nodes).map((n) => ({
            min: { x: n.position[0], y: n.position[1] },
            max: { x: n.position[0] + n.size[0], y: n.position[1] + n.size[1] },
        }));
    }

    async function cycleEdgeStyle(edgeId: string, currentStyle: string) {
        const styles = ["Cubic", "Linear", "Orthogonal"];
        const nextStyle =
            styles[(styles.indexOf(currentStyle) + 1) % styles.length];
        try {
            await invoke("set_connection_wire_style", {
                id: edgeId,
                style: nextStyle,
            });
            await onRefresh();
        } catch (err) {
            console.error(err);
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
    oncontextmenu={onContextMenu}
    ondragenter={(e) => {
        e.preventDefault();
        dragEnterCount++;
        if (!isDragOver) {
            isDragOver = true;
        }
    }}
    ondragleave={(e) => {
        e.preventDefault();
        dragEnterCount--;
        if (dragEnterCount <= 0) {
            dragEnterCount = 0;
            isDragOver = false;
        }
    }}
    ondragover={(e) => {
        e.preventDefault();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "copy";
        }
    }}
    ondrop={async (e) => {
        isDragOver = false;
        dragEnterCount = 0;
        e.preventDefault();
        const rawData = e.dataTransfer?.getData(
            "application/ferroflux-node+json",
        );
        if (rawData && canvasElement) {
            try {
                const data = JSON.parse(rawData);
                if (data.__ferroflux) {
                    // screenToWorld already handles rect.left/top
                    const { x, y } = screenToWorld(e.clientX, e.clientY);
                    const log = `Canvas: dropping ${data.name} at (${x}, ${y})`;
                    console.log(log);
                    invoke("log_js", { msg: log });
                    await invoke("add_node", { name: data.name, x, y });
                    console.log("Canvas: add_node invoked successfully");
                    await onRefresh();
                }
            } catch (err) {
                const errMsg = "Failed to parse drop data: " + err;
                console.error(errMsg);
                invoke("log_js", { msg: errMsg });
            }
        }
    }}
    style="background-position: {pan.x}px {pan.y}px; background-size: {20 *
        zoom}px {20 * zoom}px;"
>
    <div
        class="transform-layer"
        style="transform: translate({pan.x}px, {pan.y}px) scale({zoom}); transform-origin: 0 0;"
    >
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
                    selected={selectedNodeIds.has(node.id)}
                    onMouseDown={onNodeMouseDown}
                    {onPortMouseDown}
                    {onPortMouseUp}
                />
            {/each}
        {/if}

        {#if isBoxSelecting}
            {@const s = screenToWorld(boxStart.x, boxStart.y)}
            {@const e = screenToWorld(boxEnd.x, boxEnd.y)}
            <div
                class="selection-box"
                style="left: {Math.min(s.x, e.x)}px; top: {Math.min(
                    s.y,
                    e.y,
                )}px; width: {Math.abs(s.x - e.x)}px; height: {Math.abs(
                    s.y - e.y,
                )}px;"
            ></div>
        {/if}
    </div>

    {#if isDragOver}
        <div class="drop-indicator">
            <div class="drop-message">Drop to Add Node</div>
        </div>
    {/if}

    <Toolbar
        {status}
        {onDeploy}
        currentStyle={globalEdgeStyle}
        on:setStyle={(e) => setGlobalStyle(e.detail)}
        on:undo={onUndo}
        on:redo={onRedo}
    />
</div>

<svelte:window onkeydown={onKeyDown} />

<style>
    .canvas {
        position: absolute;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: #111;
        overflow: hidden;
        cursor: grab;
        background-image: radial-gradient(#333 1px, transparent 1px);
        z-index: 1; /* Ensure it's in a known stacking context */
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
        overflow: visible;
        pointer-events: none;
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
    .selection-box {
        position: absolute;
        background: rgba(96, 165, 250, 0.1);
        border: 1px solid rgba(96, 165, 250, 0.5);
        pointer-events: none;
        z-index: 1000;
    }
    .drop-indicator {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(96, 165, 250, 0.1);
        border: 4px dashed #60a5fa;
        display: flex;
        align-items: center;
        justify-content: center;
        pointer-events: none;
        z-index: 2000;
        box-sizing: border-box;
    }
    .drop-message {
        background: #111;
        border: 1px solid #333;
        padding: 12px 24px;
        border-radius: 8px;
        color: #60a5fa;
        font-weight: bold;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    }
</style>
