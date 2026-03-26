<script lang="ts">
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import { MousePointer2, Hand, MessageSquare, SquareDashed, Grid3x3 } from 'lucide-svelte';
    
    let { state: canvasState }: { state: CanvasState } = $props();
    
    // active tool state is visual for now
    let activeTool = $state('select');
</script>

<div class="absolute bottom-6 left-1/2 -translate-x-1/2 bg-bg-sidebar/90 backdrop-blur border border-border rounded-full p-1.5 shadow-lg flex items-center gap-1 z-20 select-none">
    <button class="p-2 rounded-full transition-colors {activeTool === 'select' ? 'bg-brand/20 text-brand' : 'text-text-muted hover:text-text hover:bg-white/5'}" onclick={() => activeTool = 'select'} title="Select">
        <MousePointer2 size={18} />
    </button>
    <button class="p-2 rounded-full transition-colors {activeTool === 'pan' ? 'bg-brand/20 text-brand' : 'text-text-muted hover:text-text hover:bg-white/5'}" onclick={() => activeTool = 'pan'} title="Pan Canvas (Space)">
        <Hand size={18} />
    </button>
    
    <div class="w-px h-6 bg-border mx-1"></div>
    
    <button class="p-2 rounded-full transition-colors text-text-muted hover:text-text hover:bg-white/5" onclick={async () => {
        const centerPos = canvasState.screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
        const newId = await canvasState.backend.addNode('core.comment', centerPos.x - 100, centerPos.y - 100);
        await canvasState.refreshGraph();
        canvasState.selectedNodes = new Set([newId]);
        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
    }} title="Add Comment">
        <MessageSquare size={18} />
    </button>
    <button class="p-2 rounded-full transition-colors text-text-muted hover:text-text hover:bg-white/5" onclick={async () => {
        const centerPos = canvasState.screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
        const newId = await canvasState.backend.addNode('core.group', centerPos.x - 250, centerPos.y - 200);
        await canvasState.refreshGraph();
        canvasState.selectedNodes = new Set([newId]);
        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
    }} title="Add Group">
        <SquareDashed size={18} />
    </button>
    
    <div class="w-px h-6 bg-border mx-1"></div>
    
    <button class="p-2 rounded-full text-text-muted hover:text-text hover:bg-white/5 transition-colors" title="Toggle Grid Snapping">
        <Grid3x3 size={18} />
    </button>
</div>
