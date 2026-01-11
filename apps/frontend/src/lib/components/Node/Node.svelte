<script lang="ts">
    import { useGraph } from "../../stores/graph.svelte";
    import { useSdk } from "../../context/sdk.svelte";
    import type { SerializableNode } from "../../api/adapter";

    let { node }: { node: SerializableNode } = $props();

    const graph = useGraph();
    const sdk = useSdk();

    let isDragging = $state(false);
    let startPos = { x: 0, y: 0 };
    let initialNodePos = { x: 0, y: 0 };

    function onMouseDown(e: MouseEvent) {
        if (e.button !== 0) return; // Only left click
        e.stopPropagation();

        isDragging = true;
        startPos = { x: e.clientX, y: e.clientY };
        initialNodePos = { x: node.position[0], y: node.position[1] };

        graph.selectNode(node.id, !e.shiftKey);

        window.addEventListener("mousemove", onMouseMove);
        window.addEventListener("mouseup", onMouseUp);
    }

    function onMouseMove(e: MouseEvent) {
        if (!isDragging) return;

        const dx = (e.clientX - startPos.x) / graph.scale;
        const dy = (e.clientY - startPos.y) / graph.scale;

        const newX = initialNodePos.x + dx;
        const newY = initialNodePos.y + dy;

        graph.updateNodePosition(node.id, newX, newY, false);
    }

    function onMouseUp() {
        if (!isDragging) return;
        isDragging = false;

        // Commit the final position
        graph.updateNodePosition(
            node.id,
            node.position[0],
            node.position[1],
            true,
        );

        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
    }

    async function runShadowMode() {
        console.log("Running Shadow Mode for:", node.id);
        try {
            // Placeholder: Payload and Mocks would come from UI/Inspector
            const result = await sdk.simulateNode(node.id, {}, {});
            console.log("Shadow Mode Result:", result);
            // TODO: Display result in Inspector or Overlay
        } catch (e) {
            console.error("Shadow Mode Failed:", e);
        }
    }
</script>

<div
    class="node"
    class:selected={graph.selectedNodes.has(node.id)}
    style="transform: translate({node.position[0]}px, {node
        .position[1]}px); width: {node.size[0]}px;"
    onmousedown={onMouseDown}
    role="button"
    tabindex="0"
>
    <div class="header">
        <span class="title">{node.data.name}</span>
        <button
            class="shadow-btn"
            onclick={(e) => {
                e.stopPropagation();
                runShadowMode();
            }}
            title="Run Shadow Mode">▶</button
        >
    </div>

    <div class="body">
        <div class="inputs">
            {#each node.inputs as portId}
                <div class="port input" title={`Port ${portId}`}>
                    <div class="port-dot"></div>
                </div>
            {/each}
        </div>
        <div class="spacer"></div>
        <div class="outputs">
            {#each node.outputs as portId}
                <div class="port output" title={`Port ${portId}`}>
                    <div class="port-dot"></div>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .node {
        position: absolute;
        display: flex;
        flex-direction: column;
        background: var(--node-bg);
        border: 1px solid var(--node-border);
        border-radius: 6px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        color: var(--text-primary);
        user-select: none;
        cursor: grab;
        z-index: 10;
        min-height: 80px;
    }

    .node.selected {
        border-color: var(--node-selected-border);
        box-shadow: 0 0 0 2px var(--node-selected-border);
        z-index: 20;
    }

    .header {
        background: var(--node-header-bg);
        padding: 8px 12px;
        border-top-left-radius: 5px;
        border-top-right-radius: 5px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.9em;
        font-weight: 600;
    }

    .shadow-btn {
        background: none;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        font-size: 0.8em;
        padding: 0 4px;
        transition: color 0.2s;
    }
    .shadow-btn:hover {
        color: var(--accent-color);
    }

    .body {
        padding: 10px;
        display: flex;
        justify-content: space-between;
        flex-grow: 1;
    }

    .port {
        display: flex;
        align-items: center;
        margin: 4px 0;
        height: 12px;
    }
    .port.input {
        justify-content: flex-start;
    }
    .port.output {
        justify-content: flex-end;
    }

    .port-dot {
        width: 10px;
        height: 10px;
        background: var(--port-color);
        border-radius: 50%;
        border: 1px solid var(--bg-primary);
        cursor: crosshair;
    }
    .port-dot:hover {
        background: var(--accent-color);
        transform: scale(1.2);
    }

    .inputs {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
    }
    .outputs {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
    }
    .spacer {
        flex-grow: 1;
    }
</style>
