<script lang="ts">
    import type { SerializableNode } from "$lib/types";

    let { node, selected, onMouseDown, onPortMouseDown, onPortMouseUp } =
        $props<{
            node: SerializableNode;
            selected: boolean;
            onMouseDown: (e: MouseEvent, id: string) => void;
            onPortMouseDown: (
                e: MouseEvent,
                nodeId: string,
                portId: string,
                isOutput: boolean,
            ) => void;
            onPortMouseUp: (
                e: MouseEvent,
                nodeId: string,
                portId: string,
                isOutput: boolean,
            ) => void;
        }>();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="node"
    class:selected
    onmousedown={(e) => onMouseDown(e, node.id)}
    style="transform: translate({node.position[0]}px, {node
        .position[1]}px); width: {node.size[0]}px; height: {node.size[1]}px;"
>
    <!-- Ports -->
    <div class="ports-in">
        {#each node.inputs as portId}
            <div
                class="port input"
                onmousedown={(e) => onPortMouseDown(e, node.id, portId, false)}
                onmouseup={(e) => onPortMouseUp(e, node.id, portId, false)}
            ></div>
        {/each}
    </div>
    <div class="ports-out">
        {#each node.outputs as portId}
            <div
                class="port output"
                onmousedown={(e) => onPortMouseDown(e, node.id, portId, true)}
                onmouseup={(e) => onPortMouseUp(e, node.id, portId, true)}
            ></div>
        {/each}
    </div>

    <header>{node.data?.name || "Node"}</header>
    <div class="body">
        ID: {node.uuid?.slice(0, 8) || "?"}<br />
        Type: {node.data?.node_type || "PlaygroundNode"}
    </div>
</div>

<style>
    .node {
        position: absolute;
        top: 0;
        left: 0;
        pointer-events: auto; /* Restore interactivity */
        background: #2a2a2a;
        border: 1px solid #444;
        border-radius: 6px;
        color: #eee;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
        font-size: 14px;
        display: flex;
        flex-direction: column;
        cursor: grab;
    }
    .node.selected {
        border-color: #3b82f6;
        box-shadow:
            0 0 0 2px rgba(59, 130, 246, 0.5),
            0 8px 24px rgba(0, 0, 0, 0.7);
        z-index: 100 !important;
    }
    .node:active {
        cursor: grabbing;
    }
    .node.selected header {
        background: #3b82f6;
        color: white;
        border-bottom-color: #2563eb;
    }
    header {
        background: #333;
        padding: 6px 10px;
        font-weight: 600;
        border-top-left-radius: 5px;
        border-top-right-radius: 5px;
        border-bottom: 1px solid #444;
    }
    .body {
        padding: 10px;
        flex: 1;
        color: #aaa;
        font-size: 12px;
        line-height: 1.4;
    }
    .ports-in,
    .ports-out {
        position: absolute;
        top: 0;
        bottom: 0;
        display: flex;
        flex-direction: column;
        justify-content: space-around;
        padding: 20px 0;
        z-index: 10;
    }
    .ports-in {
        left: -8px;
    }
    .ports-out {
        right: -8px;
    }
    .port {
        width: 14px;
        height: 14px;
        background: #444;
        border: 2px solid #222;
        border-radius: 50%;
        cursor: crosshair;
    }
    .port:hover {
        background: #007acc;
        scale: 1.2;
    }
</style>
