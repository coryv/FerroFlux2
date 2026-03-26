<script lang="ts">
    import { onMount, getContext } from "svelte";
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import { X, FileJson, Plus, Search, Calendar, HardDrive } from "lucide-svelte";
    import { fly } from "svelte/transition";

    let { state: canvasState, onClose }: { state: CanvasState, onClose: () => void } = $props();

    let workflows = $state<Array<{ id: string, name: string, last_modified: number, node_count: number }>>([]);
    let searchQuery = $state("");
    let isLoading = $state(true);

    let filteredWorkflows = $derived(
        workflows.filter(w => w.name.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    onMount(async () => {
        await fetchWorkflows();
    });

    async function fetchWorkflows() {
        isLoading = true;
        try {
            workflows = await canvasState.backend.listWorkflows();
            // Sort by most recent first
            workflows.sort((a, b) => b.last_modified - a.last_modified);
        } catch (e) {
            console.error("Failed to load workflows", e);
        } finally {
            isLoading = false;
        }
    }

    async function openWorkflow(id: string) {
        try {
            const jsonStr = await canvasState.backend.loadWorkflow(id);
            const data = JSON.parse(jsonStr);
            await canvasState.loadFromJson(data, id);
            onClose();
        } catch (e) {
            console.error("Failed to open workflow", e);
            alert("Failed to load workflow: " + e);
        }
    }

    async function handleNewWorkflow() {
        await canvasState.backend.newWorkflow();
        canvasState.currentWorkflowId = null;
        canvasState.currentWorkflowName = "Untitled Workflow";
        canvasState.selectedNodes.clear();
        await canvasState.refreshGraph();
        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
        onClose();
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-6" onclick={onClose}>
    <div 
        class="bg-[#0f111a] border border-white/10 rounded-xl shadow-2xl w-full max-w-4xl max-h-[85vh] flex flex-col overflow-hidden text-text"
        onclick={(e) => e.stopPropagation()}
        transition:fly={{ y: 20, duration: 250 }}
    >
        <!-- Header -->
        <div class="flex items-center justify-between p-6 border-b border-white/5 bg-white/5">
            <div>
                <h2 class="text-xl font-bold">Your Workflows</h2>
                <p class="text-sm text-text-muted mt-1">Manage and load your saved automation graphs.</p>
            </div>
            
            <button onclick={onClose} class="p-2 hover:bg-white/10 rounded-full transition-colors text-text-subtle hover:text-text">
                <X size={20} />
            </button>
        </div>

        <!-- Controls -->
        <div class="p-6 border-b border-white/5 flex items-center gap-4">
            <div class="flex-1 relative">
                <Search size={16} class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
                <input 
                    type="text" 
                    placeholder="Search workflows..." 
                    bind:value={searchQuery}
                    class="w-full bg-bg-input border border-border rounded-lg pl-9 pr-4 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-brand focus:border-brand transition-all"
                />
            </div>
            
            <button 
                onclick={handleNewWorkflow}
                class="flex items-center gap-2 px-4 py-2 bg-brand text-white font-medium text-sm rounded-lg hover:bg-brand-hover transition-colors shadow-lg shadow-brand/20 active:scale-95"
            >
                <Plus size={16} />
                New Workflow
            </button>
        </div>

        <!-- Grid -->
        <div class="flex-1 overflow-y-auto p-6 custom-scrollbar bg-bg-sidebar">
            {#if isLoading}
                <div class="flex flex-col items-center justify-center h-48 gap-3 text-text-muted">
                    <div class="w-6 h-6 border-2 border-brand border-t-transparent rounded-full animate-spin"></div>
                    <p class="text-sm font-medium">Loading workflows...</p>
                </div>
            {:else if filteredWorkflows.length === 0}
                <div class="flex flex-col items-center justify-center h-48 gap-3 text-text-muted opacity-60">
                    <FileJson size={40} className="text-white/20" />
                    <p class="text-sm font-medium">No workflows found.</p>
                </div>
            {:else}
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {#each filteredWorkflows as wf}
                        <button 
                            onclick={() => openWorkflow(wf.id)}
                            class="group relative text-left bg-bg border border-border hover:border-brand/40 rounded-xl p-5 hover:bg-white/5 transition-all text-sm group focus:outline-none focus:ring-2 focus:ring-brand flex flex-col gap-4 overflow-hidden"
                        >
                            <div class="absolute inset-x-0 -bottom-px h-px bg-gradient-to-r from-transparent via-brand/0 to-transparent group-hover:via-brand/50 transition-all duration-500"></div>
                            
                            <div class="flex items-start justify-between">
                                <div class="w-10 h-10 rounded-lg bg-brand/10 text-brand flex items-center justify-center">
                                    <FileJson size={20} />
                                </div>
                            </div>
                            
                            <div>
                                <h3 class="font-bold text-base group-hover:text-brand transition-colors line-clamp-1">{wf.name}</h3>
                                <div class="flex items-center gap-3 text-xs text-text-muted mt-2">
                                    <span class="flex items-center gap-1">
                                        <HardDrive size={12} /> {wf.node_count} nodes
                                    </span>
                                    <span class="flex items-center gap-1">
                                        <Calendar size={12} /> {new Date(wf.last_modified * 1000).toLocaleDateString()}
                                    </span>
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            {/if}
        </div>
    </div>
</div>
