<script lang="ts">
    import { executionStore } from '$lib/logic/executionStore.svelte';
    import { Play, CheckCircle2, XCircle, Clock, Copy } from 'lucide-svelte';
    import type { NodeData } from '$lib/types';
    import { CanvasState } from '$lib/logic/canvasState.svelte';

    let { node, state: canvasState }: { node: NodeData, state: CanvasState } = $props();

    let nodeState = $derived(node ? executionStore.nodeStates[node.uuid] : null);

    async function reRun() {
        if (!node) return;
        // Trigger SimulateNode Tauri command or similar 
        // This is a stub for now as described in Phase 5 docs
        await canvasState.backend.simulateNode(node.uuid, {});
    }
</script>

<div class="flex flex-col gap-4">
    <!-- Status Header -->
    <div class="p-4 rounded-lg border border-border bg-bg-sidebar/50 flex flex-col gap-2 relative overflow-hidden">
        {#if nodeState?.status === 'running'}
            <div class="absolute inset-0 bg-blue-500/5 animate-pulse"></div>
            <div class="flex items-center gap-2 text-blue-400 relative z-10">
                <Clock class="animate-spin" size={18} />
                <span class="font-semibold text-sm">Running...</span>
            </div>
        {:else if nodeState?.status === 'success'}
            <div class="absolute inset-x-0 top-0 h-1 bg-green-500/50"></div>
            <div class="flex items-center justify-between relative z-10">
                <div class="flex items-center gap-2 text-green-400">
                    <CheckCircle2 size={18} />
                    <span class="font-semibold text-sm">Success</span>
                </div>
                <div class="text-xs font-mono text-text-muted flex items-center gap-1">
                    <Clock size={12} /> {nodeState.executionMs}ms
                </div>
            </div>
        {:else if nodeState?.status === 'error'}
            <div class="absolute inset-x-0 top-0 h-1 bg-red-500/50"></div>
            <div class="flex items-center justify-between relative z-10">
                <div class="flex items-center gap-2 text-red-400">
                    <XCircle size={18} />
                    <span class="font-semibold text-sm">Failed</span>
                </div>
                <div class="text-xs font-mono text-text-muted flex items-center gap-1">
                    <Clock size={12} /> {nodeState.executionMs}ms
                </div>
            </div>
        {:else}
            <div class="flex items-center gap-2 text-text-muted relative z-10">
                <div class="w-2 h-2 rounded-full bg-border-active"></div>
                <span class="font-medium text-sm">Idle / Not executed</span>
            </div>
        {/if}

        <button 
            onclick={reRun}
            class="mt-2 flex items-center justify-center gap-2 py-1.5 px-3 rounded-md bg-white/5 hover:bg-white/10 text-xs font-semibold text-text transition-colors border border-white/5 relative z-10"
        >
            <Play size={14} /> Run Node Individually
        </button>
    </div>

    <!-- Output / Error Details -->
    {#if nodeState?.error}
        <div class="flex flex-col gap-1.5">
            <h4 class="text-xs font-semibold text-red-400 uppercase tracking-wider">Error Details</h4>
            <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-md text-red-300 font-mono text-xs overflow-x-auto">
                {nodeState.error}
            </div>
        </div>
    {/if}

    {#if nodeState?.details}
        <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
                <h4 class="text-xs font-semibold text-text-subtle uppercase tracking-wider">Output Payload</h4>
                <button class="text-text-muted hover:text-text" title="Copy JSON">
                    <Copy size={12} />
                </button>
            </div>
            
            <div class="p-3 bg-bg-input rounded-md border border-border overflow-x-auto max-h-64 custom-scrollbar relative group">
                <pre class="font-mono text-[10px] sm:text-xs text-text-muted leading-relaxed whitespace-pre font-light select-text">{JSON.stringify(nodeState.details, null, 2)}</pre>
            </div>
        </div>
    {/if}

    <!-- Empty State -->
    {#if !nodeState}
        <div class="flex flex-col items-center justify-center py-10 text-text-muted opacity-60">
            <p class="text-xs font-medium text-center max-w-[200px]">Run the workflow to capture execution traces.</p>
        </div>
    {/if}
</div>
