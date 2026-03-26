<script lang="ts">
    import { Search, Command, ArrowRight, ToyBrick, Network, Settings, X } from "lucide-svelte";
    import { fly } from "svelte/transition";
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import { getContext, tick } from "svelte";

    let canvasState: CanvasState = getContext("canvas_state");

    let isOpen = $state(false);
    let query = $state("");
    let searchInput: HTMLInputElement | undefined = $state();

    export function toggle() {
        isOpen = !isOpen;
        if (isOpen) {
            query = "";
            tick().then(() => searchInput?.focus());
        }
    }

    // Global Keydown Handler
    function handleKeydown(e: KeyboardEvent) {
        if ((e.metaKey || e.ctrlKey) && e.key === "k") {
            e.preventDefault();
            toggle();
        } else if (e.key === "Escape" && isOpen) {
            isOpen = false;
        }
    }

    // Generate search results based on templates
    let results = $derived.by(() => {
        const q = query.toLowerCase();
        let items = [];

        // Add node templates
        for (const [id, t] of Object.entries(canvasState.templates)) {
            const name = t.name || t.meta?.name || id;
            if (name.toLowerCase().includes(q) || id.toLowerCase().includes(q)) {
                items.push({
                    type: 'node',
                    icon: ToyBrick,
                    label: name,
                    description: `Add ${t.category || "Node"}`,
                    action: async () => {
                        // Place at center
                        const pos = canvasState.screenToWorld(window.innerWidth / 2, window.innerHeight / 2);
                        await canvasState.backend.addNode(id, pos.x, pos.y);
                        await canvasState.refreshGraph();
                        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
                    }
                });
            }
        }

        // Add static actions
        const actions = [
            { type: 'action', icon: Settings, label: "Settings", description: "Open App Settings", action: () => alert("Settings coming soon") },
            { type: 'action', icon: Command, label: "Keyboard Shortcuts", description: "View all shortcuts", action: () => window.dispatchEvent(new KeyboardEvent('keydown', {key: '?'})) },
            { type: 'action', icon: Network, label: "Workflows", description: "Open Workflow Manager", action: () => {} } // This could be intercepted if needed
        ];

        for (const a of actions) {
            if (a.label.toLowerCase().includes(q) || a.description.toLowerCase().includes(q)) {
                items.push(a);
            }
        }

        return items.slice(0, 8); // Max 8 results
    });

    async function selectItem(item: any) {
        await item.action();
        isOpen = false;
    }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="fixed inset-0 z-[100] bg-black/50 backdrop-blur-sm flex items-start justify-center pt-[15vh]" onclick={() => isOpen = false}>
        <div 
            class="bg-bg-sidebar shadow-2xl rounded-xl border border-white/10 w-full max-w-xl flex flex-col overflow-hidden text-text"
            onclick={(e) => e.stopPropagation()}
            transition:fly={{ y: -20, duration: 200 }}
        >
            <div class="flex items-center px-4 border-b border-white/10 relative">
                <Search size={20} class="text-text-muted absolute left-4" />
                <input 
                    bind:this={searchInput}
                    bind:value={query}
                    type="text" 
                    placeholder="Search nodes, workflows, or actions..." 
                    class="w-full bg-transparent border-none py-4 pl-8 pr-4 text-base focus:outline-none focus:ring-0 placeholder:text-text-muted placeholder:font-light"
                />
                <button onclick={() => isOpen = false} class="p-1 rounded bg-bg-input text-xs text-text-muted hover:text-text border border-border px-2 font-mono">
                    ESC
                </button>
            </div>

            <div class="p-2 flex flex-col max-h-[400px] overflow-y-auto custom-scrollbar">
                {#if results.length === 0}
                    <div class="p-4 text-center text-sm text-text-muted">No results found for "{query}"</div>
                {/if}
                
                {#each results as item}
                    <button 
                        onclick={() => selectItem(item)}
                        class="flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors text-left group"
                    >
                        <div class="w-8 h-8 rounded-md bg-white/5 flex items-center justify-center text-text-muted group-hover:text-brand group-hover:bg-brand/10 transition-colors">
                            <item.icon size={16} />
                        </div>
                        <div class="flex flex-col flex-1">
                            <span class="text-sm font-medium group-hover:text-brand transition-colors">{item.label}</span>
                            <span class="text-xs text-text-subtle">{item.description}</span>
                        </div>
                        <ArrowRight size={14} class="opacity-0 group-hover:opacity-100 text-brand -translate-x-2 group-hover:translate-x-0 transition-all" />
                    </button>
                {/each}
            </div>
            
            <div class="p-3 border-t border-white/5 bg-black/20 flex items-center justify-end gap-3 text-[10px] text-text-muted">
                <span class="flex items-center gap-1"><kbd class="px-1.5 py-0.5 rounded border border-border bg-bg-input">↑↓</kbd> to navigate</span>
                <span class="flex items-center gap-1"><kbd class="px-1.5 py-0.5 rounded border border-border bg-bg-input">↵</kbd> to select</span>
            </div>
        </div>
    </div>
{/if}
