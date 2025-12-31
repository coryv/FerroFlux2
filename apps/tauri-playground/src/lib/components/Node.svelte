<script lang="ts">
    import type { SerializableNode } from "$lib/types";

    let { node, onMouseDown } = $props<{
        node: SerializableNode;
        onMouseDown: (e: MouseEvent, id: string) => void;
    }>();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="node"
    onmousedown={(e) => onMouseDown(e, node.id)}
    style="transform: translate({node.position[0]}px, {node
        .position[1]}px); width: {node.size[0]}px; height: {node.size[1]}px;"
>
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
    .node:active {
        cursor: grabbing;
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
</style>
