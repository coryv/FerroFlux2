<script lang="ts">
    import { useGraph } from "../../stores/graph.svelte";
    import Node from "../../components/Node/Node.svelte";
    import SmartEdge from "../../components/Edge/SmartEdge.svelte";
    import type { NodeId } from "../../api/adapter";

    const graph = useGraph();

    let canvasRef: HTMLDivElement;

    // Pan & Zoom Logic
    function onWheel(e: WheelEvent) {
        if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            const zoomSensitivity = 0.001;
            const newScale = Math.min(
                Math.max(0.1, graph.scale - e.deltaY * zoomSensitivity),
                5,
            );

            // Zoom towards mouse pointer
            // simplified for now: just center zoom or zoom at cursor if possible
            // To zoom at cursor, we need to adjust pan as well.
            // Let's stick to simple zoom for now to avoid complexity in this step
            graph.scale = newScale;
        } else {
            graph.pan.x -= e.deltaX;
            graph.pan.y -= e.deltaY;
        }
    }

    // Panning with Middle Click or Space+Drag
    let isPanning = false;
    let lastMouse = { x: 0, y: 0 };

    function onMouseDown(e: MouseEvent) {
        if (e.button === 1) {
            isPanning = true;
            lastMouse = { x: e.clientX, y: e.clientY };
            e.preventDefault(); // prevent text selection
        } else {
            // Click on empty space clears selection
            if (e.target === canvasRef) {
                graph.clearSelection();
            }
        }
    }

    function onMouseMove(e: MouseEvent) {
        if (isPanning) {
            const dx = e.clientX - lastMouse.x;
            const dy = e.clientY - lastMouse.y;
            graph.pan.x += dx;
            graph.pan.y += dy;
            lastMouse = { x: e.clientX, y: e.clientY };
        }
    }

    function onMouseUp() {
        isPanning = false;
    }

    // Drag and Drop from Tray
    function onDragOver(e: DragEvent) {
        e.preventDefault();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "copy";
        }
    }

    async function onDrop(e: DragEvent) {
        e.preventDefault();
        const templateId = e.dataTransfer?.getData(
            "application/ferroflux-node",
        );
        if (templateId && canvasRef) {
            const rect = canvasRef.getBoundingClientRect();
            // Convert screen coordinates to graph coordinates
            const x = (e.clientX - rect.left - graph.pan.x) / graph.scale;
            const y = (e.clientY - rect.top - graph.pan.y) / graph.scale;

            await graph.addNode(templateId, x, y);
        }
    }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    bind:this={canvasRef}
    class="infinite-canvas"
    onwheel={onWheel}
    onmousedown={onMouseDown}
    onmousemove={onMouseMove}
    onmouseup={onMouseUp}
    onmouseleave={onMouseUp}
    ondragover={onDragOver}
    ondrop={onDrop}
    role="application"
    aria-label="Workflow Editor Canvas"
    tabindex="0"
    style="
        background-position: {graph.pan.x}px {graph.pan.y}px;
        background-size: {graph.scale * 20}px {graph.scale * 20}px;
    "
>
    <!-- Transform Container -->
    <div
        class="transform-layer"
        style="
            transform: translate({graph.pan.x}px, {graph.pan
            .y}px) scale({graph.scale});
        "
    >
        <!-- Edges Layer (SVG) -->
        <svg class="edges-layer">
            {#each Object.entries(graph.edges) as [id, edge] (id)}
                <SmartEdge {edge} />
            {/each}
        </svg>

        <!-- Nodes Layer (HTML) -->
        <div class="nodes-layer">
            {#each graph.drawOrder as nodeId (nodeId)}
                {#if graph.nodes[nodeId]}
                    <Node node={graph.nodes[nodeId]} />
                {/if}
            {/each}
        </div>
    </div>
</div>

<style>
    .infinite-canvas {
        width: 100%;
        height: 100%;
        background-color: var(--canvas-bg);
        background-image: radial-gradient(
            var(--grid-color) 1px,
            transparent 1px
        );
        overflow: hidden;
        position: relative;
        cursor: crosshair;
    }

    .transform-layer {
        transform-origin: 0 0; /* Scale from top-left, we handle offset via pan */
        width: 0;
        height: 0;
        position: absolute;
        top: 0;
        left: 0;
    }

    .edges-layer {
        position: absolute;
        top: -50000px;
        left: -50000px;
        width: 100000px;
        height: 100000px;
        pointer-events: none; /* Let clicks pass through to canvas/nodes for now, edge pointers handled in component */
        overflow: visible;
    }

    /* Re-enable pointer events for paths inside svg */
    :global(.edges-layer *) {
        pointer-events: visibleStroke;
    }

    .nodes-layer {
        position: absolute;
        top: 0;
        left: 0;
    }
</style>
