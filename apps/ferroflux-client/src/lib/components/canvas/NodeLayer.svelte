<script lang="ts">
    import Node from "$lib/components/Node.svelte";
    import { CanvasState } from "$lib/logic/canvasState.svelte";

    let { state }: { state: CanvasState } = $props();
</script>

<div class="absolute inset-0 w-full h-full">
    {#each Object.values(state.graph.nodes) as node (node.id)}
        {#if state.templates[node.data]}
            <div class="pointer-events-auto contents">
                <Node
                    {node}
                    template={state.templates[node.data]}
                    selected={state.selectedNodes.has(node.id)}
                    onPortMouseDown={(pid, e) => {
                        state.dragEdgeStart = pid;
                        e.stopPropagation();
                    }}
                    onPortMouseUp={async (pid, e) => {
                        if (
                            state.dragEdgeStart &&
                            state.dragEdgeStart !== pid
                        ) {
                            await state.backend.connectPorts(
                                state.dragEdgeStart,
                                pid,
                            );
                            state.dragEdgeStart = null;
                            state.dragEdgeCurrent = null;
                            await state.refreshGraph();
                        }
                        e.stopPropagation();
                    }}
                    onmousedown={(e) => {
                        if (!state.selectedNodes.has(node.id)) {
                            if (!e.shiftKey) {
                                state.selectedNodes = new Set([node.id]);
                            } else {
                                state.selectedNodes.add(node.id);
                                state.selectedNodes = new Set(
                                    state.selectedNodes,
                                );
                            }
                        }
                        state.draggingNode = node.id;
                        e.stopPropagation();
                    }}
                />
            </div>
        {:else}
            <!-- Fallback for unknown nodes -->
            <div
                class="absolute p-2 bg-red-900 border border-red-500 text-xs rounded pointer-events-none"
                style="left: {node.position.x}px; top: {node.position.y}px;"
            >
                Unknown: {node.data}
            </div>
        {/if}
    {/each}
</div>
