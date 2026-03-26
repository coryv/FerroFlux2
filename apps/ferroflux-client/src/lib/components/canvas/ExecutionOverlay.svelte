<script lang="ts">
    import { getContext } from 'svelte';
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    import { executionStore } from '$lib/logic/executionStore.svelte';
    import { Check, X, LoaderCircle } from 'lucide-svelte';

    const canvasState: CanvasState = getContext('canvas_state');
</script>

<div class="absolute inset-0 pointer-events-none overflow-hidden z-[45]">
    <div
        style={`transform: translate(${canvasState.offset.x}px, ${canvasState.offset.y}px) scale(${canvasState.scale}); transform-origin: 0 0;`}
        class="absolute top-0 left-0 w-full h-full"
    >
        {#each Object.entries(executionStore.nodeStates) as [nodeId, state]}
            {@const node = Object.values(canvasState.graph.nodes).find(n => n.uuid === nodeId)}
            {#if node}
                <!-- Render overlay badge above the node -->
                <div
                    class="absolute text-xs flex items-center gap-1 px-2 py-1 rounded-md shadow-lg shadow-black/20 backdrop-blur-md transition-all duration-300"
                    style={`left: ${node.position.x}px; top: ${node.position.y - 32}px;`}
                    class:bg-blue-500={state.status === 'running'}
                    class:bg-green-500={state.status === 'success'}
                    class:bg-red-500={state.status === 'error'}
                    class:text-white={true}
                    class:opacity-100={state.status !== 'idle'}
                    class:opacity-0={state.status === 'idle'}
                >
                    {#if state.status === 'running'}
                        <LoaderCircle size={12} class="animate-spin" />
                        <span class="font-medium animate-pulse">Running</span>
                    {:else if state.status === 'success'}
                        <Check size={12} />
                        <span class="font-medium">{state.executionMs}ms</span>
                    {:else if state.status === 'error'}
                        <X size={12} />
                        <span class="font-medium">Error</span>
                    {/if}
                </div>
                
                <!-- Ring highlight around the node -->
                <div
                    class="absolute w-[280px] h-32 rounded-xl pointer-events-none transition-all duration-300"
                    style={`left: ${node.position.x}px; top: ${node.position.y}px;`}
                    class:ring-2={state.status !== 'idle'}
                    class:ring-blue-500={state.status === 'running'}
                    class:ring-green-500={state.status === 'success'}
                    class:ring-red-500={state.status === 'error'}
                    class:ring-offset-2={true}
                    class:ring-offset-bg={true}
                ></div>
            {/if}
        {/each}
    </div>
</div>
