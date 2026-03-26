<script lang="ts">
    import { executionStore } from '$lib/logic/executionStore.svelte';
    import { Terminal, X, ArrowDown } from 'lucide-svelte';
    import { tick } from 'svelte';

    let isOpen = $state(false);
    let logsContainer: HTMLElement;

    // Auto-scroll when new logs arrive if already near bottom
    $effect(() => {
        if (executionStore.logs.length > 0 && isOpen && logsContainer) {
            const isNearBottom = logsContainer.scrollHeight - logsContainer.scrollTop - logsContainer.clientHeight < 50;
            if (isNearBottom) {
                tick().then(() => {
                    logsContainer.scrollTop = logsContainer.scrollHeight;
                });
            }
        }
    });

    export function toggle() {
        isOpen = !isOpen;
        if (isOpen) {
            tick().then(() => {
                if (logsContainer) logsContainer.scrollTop = logsContainer.scrollHeight;
            });
        }
    }
</script>

<!-- Floating Toggle Button -->
{#if !isOpen}
    <div class="absolute bottom-4 left-4 z-50">
        <button 
            onclick={toggle}
            class="flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-sidebar border border-border shadow-lg text-text-muted hover:text-text transition-colors"
        >
            <div class="relative">
                <Terminal size={16} />
                {#if executionStore.isRunning}
                    <span class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-brand animate-ping"></span>
                    <span class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-brand"></span>
                {/if}
            </div>
            <span class="text-xs font-medium">Logs</span>
        </button>
    </div>
{/if}

<!-- Log Panel -->
<div 
    class={`absolute bottom-0 left-0 right-0 z-50 bg-[#0f111a]/95 backdrop-blur-xl border-t border-white/10 transition-all duration-300 ease-[cubic-bezier(0.16,1,0.3,1)] flex flex-col`}
    style:height={isOpen ? "300px" : "0px"}
    style:opacity={isOpen ? "1" : "0"}
    style:pointer-events={isOpen ? "auto" : "none"}
>
    <!-- Header -->
    <div class="h-10 flex items-center justify-between px-4 border-b border-white/5 bg-black/20">
        <div class="flex items-center gap-2 text-sm font-medium text-white/80">
            <Terminal size={14} />
            Execution Logs 
            {#if executionStore.isRunning}
                <span class="px-1.5 py-0.5 rounded bg-brand/20 text-brand text-[10px] uppercase tracking-wider">Running</span>
            {/if}
        </div>
        
        <div class="flex items-center gap-1">
            <button class="p-1 text-white/50 hover:text-white rounded hover:bg-white/10" onclick={() => { executionStore.logs = []; }}>
                <span class="text-xs">Clear</span>
            </button>
            <button class="p-1 text-white/50 hover:text-white rounded hover:bg-white/10" onclick={toggle}>
                <X size={16} />
            </button>
        </div>
    </div>

    <!-- Log Stream -->
    <div bind:this={logsContainer} class="flex-1 overflow-y-auto p-4 font-mono text-xs whitespace-pre-wrap">
        {#if executionStore.logs.length === 0}
            <div class="h-full flex items-center justify-center text-white/30 italic">
                No logs available. Execute a workflow to see output.
            </div>
        {/if}
        
        {#each executionStore.logs as log}
            <div class="mb-1 flex hover:bg-white/5 px-2 py-0.5 rounded group">
                <span class="text-white/30 w-24 shrink-0 select-none">
                    {new Date(log.timestamp).toISOString().split('T')[1].slice(0, 12)}
                </span>
                
                <span class="w-12 shrink-0 select-none font-semibold uppercase" 
                      class:text-blue-400={log.level === 'info'}
                      class:text-amber-400={log.level === 'warn'}
                      class:text-red-400={log.level === 'error'}
                >
                    {log.level}
                </span>

                <span class="text-white/80 break-all">
                    {log.message}
                </span>
            </div>
        {/each}
    </div>
</div>
