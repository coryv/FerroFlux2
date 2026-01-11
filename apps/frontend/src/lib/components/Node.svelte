<script lang="ts">
    import { getTemplate } from "$lib/stores/nodeRegistry.svelte";
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

    let template = $derived(
        node.data?.template_id ? getTemplate(node.data.template_id) : undefined,
    );
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="node"
    class:selected
    onmousedown={(e) => onMouseDown(e, node.id)}
    style="transform: translate({node.position[0]}px, {node
        .position[1]}px); width: {template?.default_width ||
        node.size[0]}px; --node-color: {selected ? '#3b82f6' : '#444'};"
>
    <!-- Header -->
    <header style="background: {selected ? '#3b82f6' : '#333'}">
        <span class="title">{node.data?.name || template?.name || "Node"}</span>
        {#if template?.category}
            <span class="category">{template.category}</span>
        {/if}
    </header>

    <!-- Body with Ports -->
    <div class="body">
        <div class="io-container">
            <!-- Inputs (Left) -->
            <div class="inputs-column">
                {#each node.inputs as portId, i}
                    {@const meta = template?.inputs[i]}
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="port-row input">
                        <div
                            class="port-dot"
                            class:flow={meta?.data_type === "flow"}
                            onmousedown={(e) =>
                                onPortMouseDown(e, node.id, portId, false)}
                            onmouseup={(e) =>
                                onPortMouseUp(e, node.id, portId, false)}
                            title={meta?.data_type || "any"}
                        ></div>
                        <span class="port-label">{meta?.name || `In ${i}`}</span
                        >
                    </div>
                {/each}
            </div>

            <div class="spacer"></div>

            <!-- Outputs (Right) -->
            <div class="outputs-column">
                {#each node.outputs as portId, i}
                    {@const meta = template?.outputs[i]}
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="port-row output">
                        <span class="port-label"
                            >{meta?.name || `Out ${i}`}</span
                        >
                        <div
                            class="port-dot"
                            class:flow={meta?.data_type === "flow"}
                            onmousedown={(e) =>
                                onPortMouseDown(e, node.id, portId, true)}
                            onmouseup={(e) =>
                                onPortMouseUp(e, node.id, portId, true)}
                            title={meta?.data_type || "any"}
                        ></div>
                    </div>
                {/each}
            </div>
        </div>
        {#if !template}
            <div class="debug-info">
                ID: {node.uuid?.slice(0, 8)}
            </div>
        {/if}
    </div>
</div>

<style>
    .node {
        position: absolute;
        top: 0;
        left: 0;
        pointer-events: auto;
        background: rgba(30, 30, 35, 0.95);
        border: 1px solid var(--node-color);
        border-radius: 8px;
        color: #eee;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
        font-size: 13px;
        display: flex;
        flex-direction: column;
        cursor: grab;
        min-width: 140px;
        transition:
            border-color 0.1s,
            box-shadow 0.1s;
    }
    .node.selected {
        box-shadow:
            0 0 0 2px rgba(59, 130, 246, 0.5),
            0 8px 30px rgba(0, 0, 0, 0.6);
        z-index: 100 !important;
    }
    header {
        padding: 8px 12px;
        font-weight: 600;
        border-top-left-radius: 7px;
        border-top-right-radius: 7px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 8px;
    }
    .title {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .category {
        font-size: 9px;
        opacity: 0.6;
        text-transform: uppercase;
        font-weight: 700;
        background: rgba(0, 0, 0, 0.2);
        padding: 2px 4px;
        border-radius: 3px;
    }
    .body {
        position: relative;
        flex: 1;
        padding: 8px 0;
    }
    .io-container {
        display: flex;
        justify-content: space-between;
        gap: 16px;
        padding: 0 8px; /* Slightly inset ports for aesthetics */
    }
    .inputs-column,
    .outputs-column {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .inputs-column {
        align-items: flex-start;
    }
    .outputs-column {
        align-items: flex-end;
    }

    .port-row {
        display: flex;
        align-items: center;
        gap: 8px;
        height: 20px;
    }
    .port-dot {
        width: 10px;
        height: 10px;
        background: #777;
        border: 2px solid #333;
        border-radius: 50%;
        cursor: crosshair;
        flex-shrink: 0;
        transition:
            transform 0.1s,
            background-color 0.1s;
    }
    /* Flow ports are triangle-ish or just distinct color? standard is white usually for execution */
    .port-dot.flow {
        background: #fff;
        border-radius: 3px; /* Square/Diamond for flow */
        transform: rotate(45deg);
        width: 8px;
        height: 8px;
    }

    .port-dot:hover {
        transform: scale(1.3);
        background: #3b82f6;
    }
    .port-dot.flow:hover {
        transform: rotate(45deg) scale(1.3);
    }

    .input .port-dot {
        margin-left: -13px; /* Pull outside slightly */
    }
    .output .port-dot {
        margin-right: -13px;
    }

    .port-label {
        font-size: 11px;
        color: #bbb;
        pointer-events: none;
    }

    .debug-info {
        padding: 8px;
        font-size: 10px;
        color: #555;
    }
</style>
