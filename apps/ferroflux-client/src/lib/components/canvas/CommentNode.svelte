<script lang="ts">
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    let { node, state: canvasState }: { node: any, state: CanvasState } = $props();

    let text = $state(node.config?.text || 'New Comment');
    let color = $state(node.config?.color || '#fef3c7'); // Default yellow
    let width = $state(node.config?.width || 200);
    let height = $state(node.config?.height || 200);

    function updateConfig() {
        canvasState.backend.updateNodeConfig(node.id, 'text', text);
        canvasState.backend.updateNodeConfig(node.id, 'color', color);
        canvasState.backend.updateNodeConfig(node.id, 'width', width);
        canvasState.backend.updateNodeConfig(node.id, 'height', height);
    }
</script>

<div 
    class="absolute rounded-lg shadow-md flex flex-col pointer-events-auto group cursor-grab active:cursor-grabbing border border-black/10 transition-shadow hover:shadow-lg"
    style="left: {node.position.x}px; top: {node.position.y}px; width: {width}px; height: {height}px; background-color: {color}; color: #453A1D; z-index: -1;"
    onmousedown={(e) => {
        canvasState.selectedNodes = new Set([node.id]);
        canvasState.draggingNode = node.id;
        e.stopPropagation();
    }}
    role="presentation"
>
    <!-- Top Drag Handle area -->
    <div class="h-6 w-full flex-shrink-0 opacity-50 flex items-center px-2 pointer-events-none">
        <div class="w-full flex gap-1 items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
            <div class="h-1 w-1 rounded-full bg-black/20"></div>
            <div class="h-1 w-1 rounded-full bg-black/20"></div>
            <div class="h-1 w-1 rounded-full bg-black/20"></div>
        </div>
    </div>

    <!-- Text area -->
    <textarea
        class="w-full flex-1 bg-transparent resize-none border-none px-3 pb-3 text-xs focus:outline-none font-medium placeholder:text-black/30 custom-scrollbar-dark leading-relaxed"
        bind:value={text}
        onchange={updateConfig}
        onmousedown={(e) => e.stopPropagation()}
        placeholder="Add a comment... (Markdown supported visually in future)"
    ></textarea>
</div>

<style>
    .custom-scrollbar-dark::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar-dark::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar-dark::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.1); border-radius: 4px; }
    .custom-scrollbar-dark::-webkit-scrollbar-thumb:hover { background: rgba(0,0,0,0.2); }
</style>
