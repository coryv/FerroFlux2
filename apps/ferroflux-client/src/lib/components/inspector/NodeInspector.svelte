<script lang="ts">
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    import { fly } from 'svelte/transition';
    import { X, Settings, Database, ArrowRightCircle } from 'lucide-svelte';
    import { cn } from '$lib/utils';
    import ConfigurationPanel from './ConfigurationPanel.svelte';
    import InputsPanel from './InputsPanel.svelte';
    import OutputsPanel from './OutputsPanel.svelte';

    let { state: canvasState }: { state: CanvasState } = $props();

    let selectedNodeId = $derived(Array.from(canvasState.selectedNodes)[0]);
    let selectedNode = $derived(selectedNodeId ? canvasState.graph.nodes[selectedNodeId] : null);
    
    // Mock template fallback
    const mockTemplate = {
        meta: { name: 'HTTP Request', description: 'Make an outgoing HTTP request.' },
        interface: {
            settings: [
                { name: 'url', label: 'URL', type: 'string', required: true },
                { name: 'method', label: 'Method', type: 'select', options: ['GET', 'POST', 'PUT', 'DELETE'], default: 'GET' },
                { name: 'body', label: 'Body (JSON)', type: 'textarea', description: 'Use {{ inputs.var }}.' },
                { name: 'output_var', label: 'Output Variable', type: 'string', default: 'http_response' }
            ]
        }
    };

    let template = $derived(
        selectedNode && canvasState.templates[selectedNode.data] 
        ? canvasState.templates[selectedNode.data] 
        : mockTemplate
    );
    
    $effect(() => {
        console.log('DEBUG TEMPLATE:', template);
    });

    let isTrigger = $derived(
        (template.type === 'Trigger') || 
        (template.meta?.type === 'Trigger') ||
        (template.category === 'Triggers') // Fallback check
    );
    
    // Resizable Logic (Vertical)
    let height = $state(400); // Default height
    let isResizing = $state(false);

    function startResize(e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        isResizing = true;
    }

    function onWindowMouseMove(e: MouseEvent) {
        if (!isResizing) return;
        // Calculate new height from bottom
        const newHeight = window.innerHeight - e.clientY;
        height = Math.max(200, Math.min(newHeight, window.innerHeight - 100)); // Clamp
    }

    function onWindowMouseUp() {
        isResizing = false;
    }

    function closeInspector() {
        canvasState.selectedNodes.clear();
        canvasState.selectedNodes = new Set(); // Trigger reactivity
    }

    $effect(() => {
        if (isResizing) {
            window.addEventListener('mousemove', onWindowMouseMove);
            window.addEventListener('mouseup', onWindowMouseUp);
        } else {
            window.removeEventListener('mousemove', onWindowMouseMove);
            window.removeEventListener('mouseup', onWindowMouseUp);
        }
        return () => {
            window.removeEventListener('mousemove', onWindowMouseMove);
            window.removeEventListener('mouseup', onWindowMouseUp);
        };
    });
</script>

{#if selectedNode}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
        class="absolute bottom-0 left-0 right-0 bg-bg-sidebar border-t border-border shadow-[0_-4px_20px_rgba(0,0,0,0.3)] flex flex-col z-50 transition-[height] duration-0 ease-linear"
        style="height: {height}px;"
        onmousedown={(e) => e.stopPropagation()}
    >
        <!-- Resize Handle (Top) -->
        <div 
            class="absolute top-0 left-0 right-0 h-1 cursor-row-resize hover:bg-brand active:bg-brand transition-colors z-50 -translate-y-1/2"
            onmousedown={startResize}
        ></div>
        
        <!-- Header -->
        <div class="h-10 border-b border-border flex items-center px-4 justify-between bg-bg-sidebar shrink-0">
            <div class="flex items-center gap-3">
                <div class="flex items-center gap-2">
                    <div class="w-5 h-5 rounded bg-bg-input border border-border flex items-center justify-center text-brand">
                        <Settings size={12} />
                    </div>
                    <h3 class="font-medium text-xs text-text">{template.name || template.meta?.name}</h3>
                </div>
                <div class="h-4 w-px bg-border"></div>
                <p class="text-[10px] text-text-muted font-mono">{selectedNodeId}</p>
            </div>
            <button onclick={closeInspector} class="text-text-subtle hover:text-text p-1 hover:bg-bg-hover rounded transition-colors">
                <X size={14} />
            </button>
        </div>

        <!-- Content Columns -->
        <div class="flex-1 flex overflow-hidden">
            <!-- Col 1: Inputs (Source) - Only for Actions/Logic, not Triggers -->
            {#if !isTrigger}
                <div class="w-1/4 border-r border-border flex flex-col min-w-[200px]">
                    <div class="h-8 border-b border-border flex items-center px-3 bg-bg-sidebar/50">
                        <span class="text-[10px] font-bold text-text-subtle uppercase tracking-wider flex items-center gap-1">
                            <Database size={12} /> Inputs
                        </span>
                    </div>
                    <div class="flex-1 overflow-y-auto p-3 bg-bg">
                        <InputsPanel />
                    </div>
                </div>
            {/if}

            <!-- Col 2: Configuration (Drop Target) -->
            <div class="flex-1 flex flex-col min-w-[300px]">
                <div class="h-8 border-b border-border flex items-center px-3 bg-bg-sidebar/50">
                    <span class="text-[10px] font-bold text-text-subtle uppercase tracking-wider flex items-center gap-1">
                        <Settings size={12} /> Configuration
                    </span>
                </div>
                <div class="flex-1 overflow-y-auto p-4 bg-bg">
                    <ConfigurationPanel template={template} node={selectedNode} />
                </div>
            </div>

            <!-- Col 3: Outputs (Preview/Mapping) -->
            <div class="w-1/4 border-l border-border flex flex-col min-w-[200px] {isTrigger ? 'flex-[0.5]' : ''}">
                <div class="h-8 border-b border-border flex items-center px-3 bg-bg-sidebar/50">
                     <span class="text-[10px] font-bold text-text-subtle uppercase tracking-wider flex items-center gap-1">
                        <ArrowRightCircle size={12} /> Outputs
                    </span>
                </div>
                <div class="flex-1 overflow-y-auto p-3 bg-bg">
                    <OutputsPanel />
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    /* Add simple fly transition logic if svelte/transition is not available globally */
</style>