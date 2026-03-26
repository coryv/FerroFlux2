<script lang="ts">
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    
    let { state: canvasState }: { state: CanvasState } = $props();
    
    let vx = $derived((-canvasState.offset.x / canvasState.scale / 4000) * 100 + 50);
    let vy = $derived((-canvasState.offset.y / canvasState.scale / 4000) * 100 + 50);
    let vw = $derived((1000 / canvasState.scale / 4000) * 100);
    let vh = $derived((800 / canvasState.scale / 4000) * 100);
</script>

<div class="absolute bottom-6 right-6 w-48 h-32 bg-bg-sidebar/80 backdrop-blur border border-border rounded-lg shadow-lg z-20 overflow-hidden group select-none pointer-events-none">
    <div class="relative w-full h-full bg-bg/50">
        <!-- Render tiny proxy of nodes -->
        {#each Object.values(canvasState.graph.nodes) as node}
            <div 
                class="absolute w-2 h-1 bg-brand/50 rounded-sm"
                style="left: {(node.position.x / 4000) * 100 + 50}%; top: {(node.position.y / 4000) * 100 + 50}%;"
            ></div>
        {/each}
        
        <!-- Viewport indicator -->
        <div 
            class="absolute border border-brand/80 bg-brand/10 shadow-[0_0_0_9999px_rgba(0,0,0,0.3)] transition-all duration-75"
            style="left: {vx}%; top: {vy}%; width: {vw}%; height: {vh}%;"
        ></div>
    </div>
</div>
