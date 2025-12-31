<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { GraphState, SerializableNode } from "$lib/types";
    import Node from "./Node.svelte";

    let { graph, onRefresh } = $props<{
        graph: GraphState;
        onRefresh: () => Promise<void>;
    }>();

    let pan = $state({ x: 0, y: 0 });
    let zoom = $state(1);
    let isDraggingCanvas = $state(false);
    let draggingNodeId = $state<string | null>(null);
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
        }
    }

    async function onNodeMouseDown(e: MouseEvent, nodeId: string) {
        e.stopPropagation();
        e.preventDefault();
        draggingNodeId = nodeId;

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

    function findPortPosition(portId: string) {
        const nodes = Object.values(graph.nodes) as SerializableNode[];
        for (const node of nodes) {
            const inIdx = node.inputs.indexOf(portId);
            if (inIdx !== -1) {
                const spacing = node.size[1] / (node.inputs.length + 1);
                return {
                    x: node.position[0],
                    y: node.position[1] + spacing * (inIdx + 1),
                };
            }
            const outIdx = node.outputs.indexOf(portId);
            if (outIdx !== -1) {
                const spacing = node.size[1] / (node.outputs.length + 1);
                return {
                    x: node.position[0] + node.size[0],
                    y: node.position[1] + spacing * (outIdx + 1),
                };
            }
        }
        return null;
    }

    function getSmartOrthogonalPath(
        start: { x: number; y: number },
        end: { x: number; y: number },
        edge: { from: string; to: string },
        buffer: number = 20,
    ) {
        const outset = 20;
        const pStartReal = { x: start.x, y: start.y };
        const pStart = { x: start.x + outset, y: start.y };
        const pEndReal = { x: end.x, y: end.y };
        const pEnd = { x: end.x - outset, y: end.y };

        if (
            Math.abs(pStart.x - pEnd.x) < 1 &&
            Math.abs(pStart.y - pEnd.y) < 1
        ) {
            return [pStartReal, pEndReal];
        }

        // 1. Collect obstacles (exclude from/to nodes)
        const allNodes = Object.values(graph.nodes) as SerializableNode[];

        const obstacles = allNodes.map((n) => ({
            min: { x: n.position[0], y: n.position[1] },
            max: {
                x: n.position[0] + n.size[0],
                y: n.position[1] + n.size[1],
            },
        }));

        // 2. Generate Grid
        let xs = [pStart.x, pEnd.x];
        let ys = [pStart.y, pEnd.y];

        // Global bypass lanes
        let minX = Math.min(pStart.x, pEnd.x);
        let maxX = Math.max(pStart.x, pEnd.x);
        let minY = Math.min(pStart.y, pEnd.y);
        let maxY = Math.max(pStart.y, pEnd.y);
        for (const obs of obstacles) {
            minX = Math.min(minX, obs.min.x);
            maxX = Math.max(maxX, obs.max.x);
            minY = Math.min(minY, obs.min.y);
            maxY = Math.max(maxY, obs.max.y);
        }
        xs.push(minX - 100, maxX + 100);
        ys.push(minY - 100, maxY + 100);

        for (const obs of obstacles) {
            // Main buffer lanes
            xs.push(obs.min.x - buffer, obs.max.x + buffer);
            ys.push(obs.min.y - buffer, obs.max.y + buffer);

            // Intermediate lanes
            xs.push(obs.min.x - buffer - 20, obs.max.x + buffer + 20);
            ys.push(obs.min.y - buffer - 20, obs.max.y + buffer + 20);

            // Help with notches
            xs.push((obs.min.x + obs.max.x) / 2);
            ys.push((obs.min.y + obs.max.y) / 2);
        }

        xs = [...new Set(xs.map((x) => Math.round(x)))].sort((a, b) => a - b);
        ys = [...new Set(ys.map((y) => Math.round(y)))].sort((a, b) => a - b);

        // 3. A* Search
        const findClosest = (val: number, arr: number[]) => {
            let minDiff = Infinity;
            let idx = 0;
            for (let i = 0; i < arr.length; i++) {
                const diff = Math.abs(arr[i] - val);
                if (diff < minDiff) {
                    minDiff = diff;
                    idx = i;
                }
            }
            return idx;
        };

        const startX = findClosest(pStart.x, xs);
        const startY = findClosest(pStart.y, ys);
        const endX = findClosest(pEnd.x, xs);
        const endY = findClosest(pEnd.y, ys);

        const openSet: [number, number, number, number][] = [
            [startX, startY, 0, 1],
        ]; // x, y, fScore, dir (1: Right, 2: Left, 3: Down, 4: Up)
        const gScore: Map<string, number> = new Map();
        const cameFrom: Map<string, [number, number]> = new Map();

        gScore.set(`${startX},${startY}`, 0);

        let found = false;
        while (openSet.length > 0) {
            openSet.sort((a, b) => a[2] - b[2]);
            const [cx, cy, _f, cDir] = openSet.shift()!;

            if (cx === endX && cy === endY) {
                found = true;
                break;
            }

            const neighbors = [
                [cx + 1, cy, 1], // Right
                [cx - 1, cy, 2], // Left
                [cx, cy + 1, 3], // Down
                [cx, cy - 1, 4], // Up
            ];

            for (const [nx, ny, nDir] of neighbors) {
                if (nx < 0 || nx >= xs.length || ny < 0 || ny >= ys.length)
                    continue;

                // 180-degree turn check
                if (cDir !== 0) {
                    if (
                        (cDir === 1 && nDir === 2) ||
                        (cDir === 2 && nDir === 1)
                    )
                        continue;
                    if (
                        (cDir === 3 && nDir === 4) ||
                        (cDir === 4 && nDir === 3)
                    )
                        continue;
                }

                const nPos = { x: xs[nx], y: ys[ny] };
                const cPos = { x: xs[cx], y: ys[cy] };
                const mPos = {
                    x: (nPos.x + cPos.x) / 2,
                    y: (nPos.y + cPos.y) / 2,
                };

                let blocked = false;
                for (const obs of obstacles) {
                    // Strict boundary check
                    const isInside = (p: { x: number; y: number }) =>
                        p.x >= obs.min.x &&
                        p.x <= obs.max.x &&
                        p.y >= obs.min.y &&
                        p.y <= obs.max.y;

                    if (isInside(nPos) || isInside(mPos)) {
                        // Safety Zone Check (Docking/Undocking)
                        // Allow being inside an obstacle if we are within 5px of the start/end targets.
                        const distToStart =
                            Math.abs(nPos.x - pStart.x) +
                            Math.abs(nPos.y - pStart.y);
                        const distToEnd =
                            Math.abs(nPos.x - pEnd.x) +
                            Math.abs(nPos.y - pEnd.y);

                        if (distToStart < 5 || distToEnd < 5) {
                            continue;
                        }

                        blocked = true;
                        break;
                    }
                }
                if (blocked) continue;

                const dist =
                    Math.abs(nPos.x - cPos.x) + Math.abs(nPos.y - cPos.y);
                let stepCost = dist;

                // Proximity penalty
                for (const obs of obstacles) {
                    if (
                        nPos.x > obs.min.x - buffer &&
                        nPos.x < obs.max.x + buffer &&
                        nPos.y > obs.min.y - buffer &&
                        nPos.y < obs.max.y + buffer
                    ) {
                        stepCost += dist * 3;
                    }
                }

                // Turn penalty
                if (cDir !== 0 && cDir !== nDir) stepCost += 150;

                const tentativeG = gScore.get(`${cx},${cy}`)! + stepCost;

                if (tentativeG < (gScore.get(`${nx},${ny}`) ?? Infinity)) {
                    cameFrom.set(`${nx},${ny}`, [cx, cy]);
                    gScore.set(`${nx},${ny}`, tentativeG);
                    const fScore =
                        tentativeG +
                        Math.abs(nPos.x - xs[endX]) +
                        Math.abs(nPos.y - ys[endY]);
                    openSet.push([nx, ny, fScore, nDir]);
                }
            }
        }

        if (found) {
            const path = [pEndReal, pEnd];
            let curr: [number, number] = [endX, endY];
            while (cameFrom.has(`${curr[0]},${curr[1]}`)) {
                path.push({ x: xs[curr[0]], y: ys[curr[1]] });
                curr = cameFrom.get(`${curr[0]},${curr[1]}`)!;
            }
            path.push(pStart);
            path.push(pStartReal);
            path.reverse();

            // Simplified
            const simplified = [path[0]];
            for (let i = 1; i < path.length - 1; i++) {
                const prev = simplified[simplified.length - 1];
                const next = path[i + 1];
                if (
                    !(
                        (prev.x === path[i].x && path[i].x === next.x) ||
                        (prev.y === path[i].y && path[i].y === next.y)
                    )
                ) {
                    simplified.push(path[i]);
                }
            }
            simplified.push(path[path.length - 1]);
            return simplified;
        }

        // Fallback
        const midX = (pStart.x + pEnd.x) / 2;
        return [
            pStartReal,
            pStart,
            { x: midX, y: pStart.y },
            { x: midX, y: pEnd.y },
            pEnd,
            pEndReal,
        ];
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
                {#each getEdges(graph) as edge (edge.id)}
                    {@const start = findPortPosition(edge.from)}
                    {@const end = findPortPosition(edge.to)}
                    {#if start && end}
                        {#if edge.style === "Cubic"}
                            {@const dx = end.x - start.x}
                            {@const dy = end.y - start.y}
                            {@const dist = Math.sqrt(dx * dx + dy * dy)}
                            {@const controlDist = Math.min(dist * 0.5, 150)}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <path
                                d="M {start.x} {start.y} C {start.x +
                                    controlDist} {start.y}, {end.x -
                                    controlDist} {end.y}, {end.x} {end.y}"
                                stroke="#666"
                                stroke-width="2"
                                fill="none"
                                class="edge-path"
                                role="button"
                                tabindex="0"
                                onclick={() =>
                                    cycleEdgeStyle(edge.id, edge.style)}
                            />
                        {:else if edge.style === "Orthogonal"}
                            {@const points = getSmartOrthogonalPath(
                                start,
                                end,
                                edge,
                            )}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <path
                                d="M {points[0].x} {points[0].y} {points
                                    .slice(1)
                                    .map((p) => `L ${p.x} ${p.y}`)
                                    .join(' ')}"
                                stroke="#666"
                                stroke-width="2"
                                fill="none"
                                class="edge-path"
                                role="button"
                                tabindex="0"
                                onclick={() =>
                                    cycleEdgeStyle(edge.id, edge.style)}
                            />
                        {:else}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <line
                                x1={start.x}
                                y1={start.y}
                                x2={end.x}
                                y2={end.y}
                                stroke="#666"
                                stroke-width="2"
                                class="edge-path"
                                role="button"
                                tabindex="0"
                                onclick={() =>
                                    cycleEdgeStyle(edge.id, edge.style)}
                            />
                        {/if}
                    {/if}
                {/each}
            {/if}

            <!-- Temporary Edge -->
            {#if connectingFrom && tempEdgeEnd}
                {@const start = findPortPosition(connectingFrom.portId)}
                {#if start}
                    {@const dx = tempEdgeEnd.x - start.x}
                    {@const dy = tempEdgeEnd.y - start.y}
                    {@const dist = Math.sqrt(dx * dx + dy * dy)}
                    {@const controlDist = Math.min(dist * 0.5, 150)}
                    <path
                        d="M {start.x} {start.y} C {start.x +
                            controlDist} {start.y}, {tempEdgeEnd.x -
                            controlDist} {tempEdgeEnd.y}, {tempEdgeEnd.x} {tempEdgeEnd.y}"
                        stroke="#007acc"
                        stroke-width="2"
                        stroke-dasharray="4"
                        fill="none"
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
    .edge-path {
        cursor: pointer;
        pointer-events: visibleStroke;
        transition: stroke 0.2s;
    }
    .edge-path:hover {
        stroke: #999;
        stroke-width: 3;
    }
</style>
