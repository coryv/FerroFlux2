<script lang="ts">
    import NodeList from "$lib/components/NodeList.svelte";
    import InfiniteCanvas from "$lib/components/InfiniteCanvas.svelte";
    import LogPanel from "$lib/components/panels/LogPanel.svelte";
    import WorkflowManager from "$lib/components/panels/WorkflowManager.svelte";
    import CommandPalette from "$lib/components/CommandPalette.svelte";
    import ShortcutsPanel from "$lib/components/panels/ShortcutsPanel.svelte";
    import { getContext, onMount } from "svelte";
    import type { CanvasState } from "$lib/logic/canvasState.svelte";

    const canvasState: CanvasState = getContext('canvas_state');

    let isWorkflowManagerOpen = $state(false);

    onMount(() => {
        const handler = () => isWorkflowManagerOpen = true;
        window.addEventListener('ferroflux:open-workflow-manager', handler);
        return () => window.removeEventListener('ferroflux:open-workflow-manager', handler);
    });
</script>

<div class="flex h-full w-full overflow-hidden relative">
    <NodeList />
    
    <!-- Canvas Area (Right Pane) -->
    <main class="flex-1 relative overflow-hidden bg-bg z-0 focus:outline-none">
        <InfiniteCanvas />
        <LogPanel />
    </main>

    {#if isWorkflowManagerOpen}
        <WorkflowManager 
            state={canvasState} 
            onClose={() => isWorkflowManagerOpen = false} 
        />
    {/if}

    <CommandPalette />
    <ShortcutsPanel />
</div>
