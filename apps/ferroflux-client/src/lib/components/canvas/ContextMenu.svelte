<script lang="ts">
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    import { fly } from 'svelte/transition';
    import { Copy, ClipboardPaste, CopyPlus, Trash2, Group, Lock } from 'lucide-svelte';

    let { x, y, state, onClose }: { x: number, y: number, state: CanvasState, onClose: () => void } = $props();

    let hasSelection = $derived(state.selectedNodes.size > 0);
    
    function wrap(action: () => Promise<void> | void) {
        return async () => {
            await action();
            onClose();
        };
    }
</script>

<div 
    class="fixed z-50 min-w-48 bg-bg-sidebar/95 backdrop-blur-md border border-border rounded-lg shadow-xl py-1"
    style="left: {x}px; top: {y}px;"
    transition:fly={{ y: 5, duration: 150 }}
>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    
    <button 
        class="w-full px-3 py-1.5 text-xs text-left flex items-center justify-between hover:bg-brand/20 transition-colors disabled:opacity-50 disabled:hover:bg-transparent"
        disabled={!hasSelection}
        onclick={wrap(() => state.copy())}
    >
        <span class="flex items-center gap-2 text-text"><Copy size={14} class="text-text-muted" /> Copy</span>
        <span class="text-[10px] text-text-subtle font-mono">⌘C</span>
    </button>

    <button 
        class="w-full px-3 py-1.5 text-xs text-left flex items-center justify-between hover:bg-brand/20 transition-colors disabled:opacity-50 disabled:hover:bg-transparent"
        disabled={state.clipboard.length === 0}
        onclick={wrap(() => state.paste())}
    >
        <span class="flex items-center gap-2 text-text"><ClipboardPaste size={14} class="text-text-muted" /> Paste</span>
        <span class="text-[10px] text-text-subtle font-mono">⌘V</span>
    </button>
    
    <button 
        class="w-full px-3 py-1.5 text-xs text-left flex items-center justify-between hover:bg-brand/20 transition-colors disabled:opacity-50 disabled:hover:bg-transparent"
        disabled={!hasSelection}
        onclick={wrap(() => state.duplicate())}
    >
        <span class="flex items-center gap-2 text-text"><CopyPlus size={14} class="text-text-muted" /> Duplicate</span>
        <span class="text-[10px] text-text-subtle font-mono">⌘D</span>
    </button>
    
    <div class="h-px bg-border/50 my-1"></div>
    
    <button 
        class="w-full px-3 py-1.5 text-xs text-left flex items-center justify-between hover:bg-status-error/20 hover:text-status-error transition-colors disabled:opacity-50 disabled:hover:bg-transparent"
        disabled={!hasSelection}
        onclick={wrap(async () => {
            for (const id of state.selectedNodes) {
                await state.backend.deleteNode(id);
            }
            state.selectedNodes.clear();
            await state.refreshGraph();
            window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
        })}
    >
        <span class="flex items-center gap-2 text-[inherit]"><Trash2 size={14} class="opacity-70" /> Delete</span>
        <span class="text-[10px] text-[inherit] font-mono opacity-60">Del</span>
    </button>
</div>
