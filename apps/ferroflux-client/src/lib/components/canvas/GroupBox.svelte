<script lang="ts">
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    let { node, state: canvasState }: { node: any, state: CanvasState } = $props();

    let title = $state(node.config?.title || 'Group');
    let color = $state(node.config?.color || '#3b82f6'); // Brand blue
    let width = $state(node.config?.width || 500);
    let height = $state(node.config?.height || 400);

    function updateConfig() {
        canvasState.backend.updateNodeConfig(node.id, 'title', title);
        canvasState.backend.updateNodeConfig(node.id, 'color', color);
        canvasState.backend.updateNodeConfig(node.id, 'width', width);
        canvasState.backend.updateNodeConfig(node.id, 'height', height);
    }
</script>

<div 
    class="absolute rounded-lg border-2 pointer-events-none group transition-colors"
    style="left: {node.position.x}px; top: {node.position.y}px; width: {width}px; height: {height}px; border-color: {color}40; background-color: {color}05; z-index: -2;"
    class:border-opacity-100={canvasState.selectedNodes.has(node.id)}
>
    <!-- Header (pointer-events-auto for dragging) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="absolute top-0 left-0 right-0 h-8 flex items-center px-3 pointer-events-auto cursor-grab active:cursor-grabbing rounded-t-[6px] backdrop-blur-md transition-colors"
        style="background-color: {color}20;"
        onmousedown={(e) => {
            // Select group and all children inside
            let newSelection = new Set([node.id]);
            for (const [id, n] of Object.entries(canvasState.graph.nodes)) {
                if (id !== node.id && 
                    n.position.x >= node.position.x && 
                    n.position.x <= node.position.x + width && 
                    n.position.y >= node.position.y && 
                    n.position.y <= node.position.y + height) {
                    newSelection.add(id);
                }
            }
            canvasState.selectedNodes = newSelection;
            canvasState.draggingNode = node.id;
            e.stopPropagation();
        }}
    >
        <input
            type="text"
            class="bg-transparent border-none text-text font-bold text-xs focus:outline-none w-full placeholder:text-white/50"
            bind:value={title}
            onchange={updateConfig}
            onmousedown={(e) => e.stopPropagation()}
            style="color: {color}; filter: brightness(1.5);"
        />
    </div>
</div>
