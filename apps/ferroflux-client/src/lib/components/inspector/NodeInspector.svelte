<script lang="ts">
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    import { fly } from 'svelte/transition';
    import { X, Settings, Database, ArrowRightCircle, Play, SlidersHorizontal, Trash2 } from 'lucide-svelte';
    import { cn } from '$lib/utils';
    import ConfigurationPanel from './ConfigurationPanel.svelte';
    import InputsPanel from './InputsPanel.svelte';
    import OutputsPanel from './OutputsPanel.svelte';

    let { state: canvasState }: { state: CanvasState } = $props();

    let selectedNodeId = $derived(Array.from(canvasState.selectedNodes)[0]);
    let selectedNode = $derived(selectedNodeId ? canvasState.graph.nodes[selectedNodeId] : null);
    
    let template = $derived(selectedNode && canvasState.templates[selectedNode.data] ? canvasState.templates[selectedNode.data] : null);
    
    let isTrigger = $derived(template && (template.type === 'Trigger' || template.meta?.type === 'Trigger' || template.category === 'Triggers'));
    
    // Tab state
    let activeTab = $state<'config'|'inputs'|'outputs'|'execution'>('config');

    $effect(() => {
        // Reset tab on node change
        if (selectedNodeId) {
            activeTab = 'config';
        }
    });

    function closeInspector() {
        canvasState.selectedNodes.clear();
        canvasState.selectedNodes = new Set(); // Trigger reactivity
    }

    async function deleteNode() {
        if (!selectedNodeId) return;
        if (confirm('Delete this node?')) {
            await canvasState.backend.deleteNode(selectedNodeId);
            canvasState.selectedNodes.clear();
            canvasState.selectedNodes = new Set();
            await canvasState.refreshGraph();
            window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
        }
    }
</script>

{#if selectedNode && template}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div 
        class="absolute top-14 right-0 bottom-0 w-[400px] bg-bg-sidebar border-l border-border shadow-2xl flex flex-col z-40"
        transition:fly={{ x: 400, duration: 250, opacity: 1 }}
        onmousedown={(e) => e.stopPropagation()}
    >
        <!-- Header -->
        <div class="p-4 border-b border-border flex flex-col gap-3 bg-bg-sidebar shrink-0">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <div class="w-6 h-6 rounded bg-brand flex items-center justify-center text-white">
                        <Settings size={14} />
                    </div>
                    <div class="flex flex-col">
                        <h3 class="font-bold text-sm text-text">{template.name || template.meta?.name}</h3>
                        <p class="text-[10px] text-text-muted font-mono truncate w-32">{selectedNodeId}</p>
                    </div>
                </div>
                <div class="flex items-center gap-1">
                    <button onclick={deleteNode} class="text-text-subtle hover:text-status-error p-1.5 hover:bg-status-error/10 rounded transition-colors" title="Delete Node">
                        <Trash2 size={16} />
                    </button>
                    <button onclick={closeInspector} class="text-text-subtle hover:text-text p-1.5 hover:bg-bg-hover rounded transition-colors" title="Close Panel">
                        <X size={16} />
                    </button>
                </div>
            </div>
            
            <!-- Tabs -->
            <div class="flex items-center gap-1 bg-bg-input p-1 rounded-md">
                <button 
                    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded text-xs font-medium transition-colors {activeTab === 'config' ? 'bg-bg-sidebar text-text shadow-sm' : 'text-text-subtle hover:text-text'}"
                    onclick={() => activeTab = 'config'}
                >
                    <SlidersHorizontal size={14} /> Config
                </button>
                {#if !isTrigger}
                <button 
                    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded text-xs font-medium transition-colors {activeTab === 'inputs' ? 'bg-bg-sidebar text-text shadow-sm' : 'text-text-subtle hover:text-text'}"
                    onclick={() => activeTab = 'inputs'}
                >
                    <Database size={14} /> Inputs
                </button>
                {/if}
                <button 
                    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded text-xs font-medium transition-colors {activeTab === 'outputs' ? 'bg-bg-sidebar text-text shadow-sm' : 'text-text-subtle hover:text-text'}"
                    onclick={() => activeTab = 'outputs'}
                >
                    <ArrowRightCircle size={14} /> Outputs
                </button>
                <button 
                    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded text-xs font-medium transition-colors {activeTab === 'execution' ? 'bg-bg-sidebar text-text shadow-sm' : 'text-text-subtle hover:text-text'}"
                    onclick={() => activeTab = 'execution'}
                >
                    <Play size={14} /> Exec
                </button>
            </div>
        </div>

        <!-- Content Area -->
        <div class="flex-1 overflow-y-auto bg-bg p-4 custom-scrollbar">
            {#if activeTab === 'config'}
                <ConfigurationPanel template={template} node={selectedNode} state={canvasState} />
            {:else if activeTab === 'inputs'}
                <InputsPanel />
            {:else if activeTab === 'outputs'}
                <OutputsPanel />
            {:else if activeTab === 'execution'}
                <div class="flex flex-col items-center justify-center h-full text-text-muted gap-2 opacity-50">
                    <Play size={32} />
                    <p class="text-sm">Execution Engine Disabled</p>
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.2);
    }
</style>