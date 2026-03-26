<script lang="ts">
    import { onMount } from "svelte";
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import ConnectionLayer from "$lib/components/canvas/ConnectionLayer.svelte";
    import NodeLayer from "$lib/components/canvas/NodeLayer.svelte";
    import SelectionOverlay from "$lib/components/canvas/SelectionOverlay.svelte";
    import NodeInspector from "$lib/components/inspector/NodeInspector.svelte";

    // Initialize State
    const canvasState = new CanvasState();

    onMount(() => {
        canvasState.refreshGraph();
        const handler = () => canvasState.refreshGraph();
        window.addEventListener("ferroflux:graph-change", handler);
        return () => {
            window.removeEventListener("ferroflux:graph-change", handler);
        };
    });

    // --- Interaction Handlers (Delegated to State) ---
    // Since listeners are on elements, we handle events here and manipulate state.

    function onWheel(e: WheelEvent) {
        e.preventDefault();
        if (e.ctrlKey || e.metaKey) {
            // ZOOM
            const zoomSensitivity = 0.002;
            const delta = -e.deltaY * zoomSensitivity;
            const newScale = Math.min(Math.max(0.1, canvasState.scale + delta), 5);

            const rect =
                e.currentTarget instanceof HTMLElement
                    ? e.currentTarget.getBoundingClientRect()
                    : { left: 0, top: 0 };
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;

            const worldBefore = canvasState.screenToWorld(mouseX, mouseY);
            canvasState.scale = newScale;
            // Adjust offset
            canvasState.offset.x = mouseX - worldBefore.x * canvasState.scale;
            canvasState.offset.y = mouseY - worldBefore.y * canvasState.scale;
        } else {
            // PAN
            canvasState.offset.x -= e.deltaX;
            canvasState.offset.y -= e.deltaY;
        }
    }

    async function onKeyDown(e: KeyboardEvent) {
        if (
            document.activeElement?.tagName === "INPUT" ||
            document.activeElement?.tagName === "TEXTAREA"
        )
            return;
        const isCmd = e.ctrlKey || e.metaKey;

        // DELETE
        if (e.key === "Backspace" || e.key === "Delete") {
            if (canvasState.selectedNodes.size > 0 || canvasState.selectedEdges.size > 0) {
                for (const id of canvasState.selectedNodes)
                    await canvasState.backend.deleteNode(id);
                for (const id of canvasState.selectedEdges)
                    await canvasState.backend.deleteConnection(id);

                canvasState.selectedNodes = new Set();
                canvasState.selectedEdges = new Set();
                await canvasState.refreshGraph();
                window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
            }
        }

        // SELECT ALL
        if (isCmd && e.key === "a") {
            e.preventDefault();
            canvasState.selectedNodes = new Set(Object.keys(canvasState.graph.nodes));
        }

        // Copy/Paste Logic could be moved to State class but ok here for now
        // TODO: Implement Copy/Paste using State access
    }

    function onCanvasMouseDown(event: MouseEvent) {
        if (event.button === 0 && !canvasState.draggingNode && !canvasState.dragEdgeStart) {
            const container = event.currentTarget as HTMLElement;
            const rect = container.getBoundingClientRect();
            const x = event.clientX - rect.left;
            const y = event.clientY - rect.top;

            canvasState.selectionStart = { x, y };
            canvasState.selectionCurrent = { x, y };

            if (!event.shiftKey) {
                canvasState.selectedNodes = new Set();
                canvasState.selectedEdges = new Set();
            }
        }
    }

    function onCanvasMouseMove(event: MouseEvent) {
        const container = event.currentTarget as HTMLElement;
        const rect = container.getBoundingClientRect();
        const mouseX = event.clientX - rect.left;
        const mouseY = event.clientY - rect.top;

        canvasState.mouseWorldPos = canvasState.screenToWorld(mouseX, mouseY);

        if (canvasState.draggingNode) {
            const node = canvasState.graph.nodes[canvasState.draggingNode];
            if (node) {
                node.position.x += event.movementX / canvasState.scale;
                node.position.y += event.movementY / canvasState.scale;
            }
        }

        if (canvasState.dragEdgeStart) {
            canvasState.dragEdgeCurrent = canvasState.screenToWorld(mouseX, mouseY);
        }

        if (canvasState.selectionStart) {
            canvasState.selectionCurrent = { x: mouseX, y: mouseY };
        }
    }

    async function onCanvasMouseUp(event: MouseEvent) {
        // 1. End Node Drag
        if (canvasState.draggingNode) {
            const node = canvasState.graph.nodes[canvasState.draggingNode];
            if (node) {
                await canvasState.backend.updateNodePosition(
                    canvasState.draggingNode,
                    node.position.x,
                    node.position.y,
                );
            }
            canvasState.draggingNode = null;
        }

        // 2. End Edge Drag
        if (canvasState.dragEdgeStart) {
            canvasState.dragEdgeStart = null;
            canvasState.dragEdgeCurrent = null;
        }

        // 3. End Selection
        if (canvasState.selectionStart && canvasState.selectionCurrent) {
            const startWorld = canvasState.screenToWorld(
                canvasState.selectionStart.x,
                canvasState.selectionStart.y,
            );
            const endWorld = canvasState.screenToWorld(
                canvasState.selectionCurrent.x,
                canvasState.selectionCurrent.y,
            );

            canvasState.handleSelection(
                startWorld.x,
                startWorld.y,
                endWorld.x,
                endWorld.y,
            );

            canvasState.selectionStart = null;
            canvasState.selectionCurrent = null;
        }
    }

    // Drag N Drop
    function onDragOver(e: DragEvent) {
        e.preventDefault();
        e.dataTransfer!.dropEffect = "copy";
        canvasState.isDraggingOver = true;
    }

    async function onDrop(e: DragEvent) {
        e.preventDefault();
        canvasState.isDraggingOver = false;

        // Check if this is a variable drop (handled by inputs)
        if (e.dataTransfer?.types.includes('application/ferroflux-var')) {
            return;
        }

        const templateId = e.dataTransfer?.getData("text/plain");
        if (templateId) {
            const rect =
                e.currentTarget instanceof HTMLElement
                    ? e.currentTarget.getBoundingClientRect()
                    : { left: 0, top: 0 };
            const worldPos = canvasState.screenToWorld(
                e.clientX - rect.left,
                e.clientY - rect.top,
            );
            const newNodeId = await canvasState.backend.addNode(templateId, worldPos.x, worldPos.y);
            await canvasState.refreshGraph();
            
            // Auto-select the new node
            canvasState.selectedNodes = new Set([newNodeId]);
            
            window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
        }
    }
</script>

<svelte:window onkeydown={onKeyDown} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    class="relative w-full h-full bg-bg overflow-hidden select-none"
    class:bg-bg-active={canvasState.isDraggingOver}
    role="application"
    ondragover={onDragOver}
    ondragleave={() => (canvasState.isDraggingOver = false)}
    ondrop={onDrop}
    onmousedown={onCanvasMouseDown}
    onmousemove={onCanvasMouseMove}
    onmouseup={onCanvasMouseUp}
    onwheel={onWheel}
>
    <!-- Background Grid -->
    <div
        class="absolute inset-0 pointer-events-none opacity-20 bg-grid-pattern"
        style="background-position: {canvasState.offset.x}px {canvasState.offset
            .y}px; background-size: {40 * canvasState.scale}px {40 * canvasState.scale}px;"
    ></div>

    {#if Object.keys(canvasState.graph.nodes).length === 0}
        <div
            class="absolute inset-0 flex items-center justify-center pointer-events-none opacity-10"
        >
            <h1 class="text-3xl font-mono">Infinite Canvas</h1>
        </div>
    {/if}

    <!-- Viewport Container -->
    <div
        style="transform: translate({canvasState.offset.x}px, {canvasState.offset
            .y}px) scale({canvasState.scale}); transform-origin: 0 0; width: 100%; height: 100%; pointer-events: none;"
    >
        <ConnectionLayer state={canvasState} />
        <NodeLayer state={canvasState} />
    </div>

    <SelectionOverlay state={canvasState} />
    
    <NodeInspector state={canvasState} />
</div>

<style>
    .bg-grid-pattern {
        background-size: 40px 40px;
        background-image: linear-gradient(to right, theme('colors.border.subtle') 1px, transparent 1px),
            linear-gradient(to bottom, theme('colors.border.subtle') 1px, transparent 1px);
    }
</style>