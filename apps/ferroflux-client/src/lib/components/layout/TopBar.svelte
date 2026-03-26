<script lang="ts">
    import { isSidebarOpen } from '$lib/stores';
    import { Menu, Play, Square, Save, Undo, Redo, ZoomIn, ZoomOut, Expand } from 'lucide-svelte';
    import { executionStore } from '$lib/logic/executionStore.svelte';
    import { getContext } from 'svelte';
    import type { CanvasState } from '$lib/logic/canvasState.svelte';
    
    const canvasState: CanvasState = getContext('canvas_state');

    function toggle() {
        isSidebarOpen.update(v => !v);
    }
</script>

<header class="h-14 border-b border-border flex items-center px-4 bg-bg select-none gap-4 z-20" data-tauri-drag-region>
    <!-- Left: Navigation / Sidebar Toggle -->
    <div class="flex items-center">
        {#if !$isSidebarOpen}
            <button onclick={toggle} class="p-1.5 hover:bg-bg-hover rounded mr-3 text-text-subtle hover:text-text transition-colors">
                <Menu size={18} />
            </button>
        {/if}
        
        <!-- Breadcrumbs -->
        <div class="flex items-center text-sm font-medium">
            <span class="text-text-muted hover:text-text cursor-pointer transition-colors">Workflows</span>
            <span class="mx-2 text-border-active">/</span>
            <input 
                type="text" 
                bind:value={canvasState.currentWorkflowName} 
                class="bg-transparent text-text border-none focus:outline-none focus:ring-1 focus:ring-brand rounded px-1 w-48 transition-all" 
            />
        </div>
    </div>

    <!-- Center: Execution Controls -->
    <div class="absolute left-1/2 -translate-x-1/2 flex items-center bg-bg-sidebar border border-border rounded-full p-1 shadow-sm">
        {#if !executionStore.isRunning}
        <button onclick={() => executionStore.runWorkflow()} class="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-brand/10 text-brand hover:bg-brand/20 transition-colors">
            <Play size={14} fill="currentColor" />
            Execute
        </button>
        {:else}
        <button onclick={() => executionStore.stop()} class="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold text-red-400 hover:text-red-300 hover:bg-red-400/10 transition-colors">
            <Square size={14} fill="currentColor" />
            Stop
        </button>
        {/if}
    </div>

    <!-- Right: Actions & View Controls -->
    <div class="ml-auto flex items-center gap-2">
        <div class="flex items-center text-text-muted border-r border-border pr-2 mr-1">
            <button class="p-1.5 hover:bg-bg-hover rounded hover:text-text transition-colors" title="Undo (⌘Z)">
                <Undo size={16} />
            </button>
            <button class="p-1.5 hover:bg-bg-hover rounded hover:text-text transition-colors opacity-50 cursor-not-allowed" title="Redo (⌘⇧Z)">
                <Redo size={16} />
            </button>
        </div>

        <div class="flex items-center text-text-muted border-r border-border pr-2 mr-1">
            <button class="p-1.5 hover:bg-bg-hover rounded hover:text-text transition-colors" title="Zoom Out (-)">
                <ZoomOut size={16} />
            </button>
            <span class="text-xs font-mono w-12 text-center">100%</span>
            <button class="p-1.5 hover:bg-bg-hover rounded hover:text-text transition-colors" title="Zoom In (+)">
                <ZoomIn size={16} />
            </button>
            <button class="p-1.5 hover:bg-bg-hover rounded hover:text-text transition-colors ml-1" title="Fit to View (⌘1)">
                <Expand size={14} />
            </button>
        </div>
        
        <button 
            onclick={async () => {
                try {
                    await canvasState.backend.saveWorkflow(canvasState.currentWorkflowName);
                    // could show a toast here
                } catch (e) {
                    console.error("Save failed:", e);
                }
            }}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold bg-white/5 text-text hover:bg-white/10 transition-colors border border-white/10 active:scale-95"
        >
            <Save size={14} />
            Save
        </button>
    </div>
</header>
