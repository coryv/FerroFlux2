<script lang="ts">
    import type { GraphState, SerializableNode } from "$lib/types";
    import Node from "./Node.svelte";

    let { graph } = $props<{ graph: GraphState }>();

    let pan = $state({ x: 0, y: 0 });
    let zoom = $state(1);
    let isDraggingCanvas = $state(false);
    let draggingNodeId = $state<string | null>(null);

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        const zoomSpeed = 0.001;
        const newZoom = Math.max(0.1, Math.min(5, zoom - e.deltaY * zoomSpeed));
        zoom = newZoom;
    }

    function onMouseDown(e: MouseEvent) {
        if (e.button === 0 || e.button === 1) {
            isDraggingCanvas = true;
        }
    }

    function onNodeMouseDown(e: MouseEvent, nodeId: string) {
        e.stopPropagation();
        draggingNodeId = nodeId;
    }

    function onMouseMove(e: MouseEvent) {
        if (draggingNodeId && graph.nodes[draggingNodeId]) {
            const dx = e.movementX / zoom;
            const dy = e.movementY / zoom;

            // Mutating the prop directly for now (client-side visual update)
            // Ideally this would emit an event, but for a playground this is efficient Svelte 5 reactivity
            const node = graph.nodes[draggingNodeId];
            node.position[0] += dx;
            node.position[1] += dy;
        } else if (isDraggingCanvas) {
            pan.x += e.movementX;
            pan.y += e.movementY;
        }
    }

    function onMouseUp() {
        isDraggingCanvas = false;
        draggingNodeId = null;
    }

    function getNodes(g: GraphState): SerializableNode[] {
        if (!g || !g.nodes) return [];
        return Object.values(g.nodes);
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
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
        {#if graph}
            {#each getNodes(graph) as node (node.id)}
                <Node {node} onMouseDown={onNodeMouseDown} />
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
</style>
