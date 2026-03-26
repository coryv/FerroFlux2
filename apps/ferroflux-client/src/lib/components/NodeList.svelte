<script lang="ts">
    import { getBackend } from "$lib/context/backend.svelte";
    import { onMount } from "svelte";
    import { getCategoryColors } from "$lib/utils/nodeColors";
    import { getNodeIcon } from "$lib/utils/nodeIcons";
    import { Search, ChevronDown, ChevronRight } from "lucide-svelte";
    import type { NodeMetadata } from "$lib/types";

    const backend = getBackend();

    let templates: NodeMetadata[] = $state([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    let searchQuery = $state("");
    let collapsedCategories = $state<Set<string>>(new Set());

    onMount(async () => {
        try {
            const data = await backend.getTemplates();
            // Sort templates alphabetically to be nice
            templates = data.sort((a: NodeMetadata, b: NodeMetadata) => a.name.localeCompare(b.name));
        } catch (e: any) {
            error = e.message || "Failed to load templates";
            console.error(e);
        } finally {
            loading = false;
        }
    });

    let filteredTemplates = $derived(
        templates.filter(t => 
            t.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
            (t.description?.toLowerCase().includes(searchQuery.toLowerCase())) ||
            t.category.toLowerCase().includes(searchQuery.toLowerCase())
        )
    );

    let groupedTemplates = $derived.by(() => {
        const groups: Record<string, NodeMetadata[]> = {};
        for (const t of filteredTemplates) {
            const cat = t.category || "Uncategorized";
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(t);
        }
        return groups;
    });

    function toggleCategory(cat: string) {
        if (collapsedCategories.has(cat)) {
            collapsedCategories.delete(cat);
        } else {
            collapsedCategories.add(cat);
        }
        collapsedCategories = new Set(collapsedCategories);
    }
</script>

<div class="w-72 border-r border-border bg-bg-sidebar h-full flex flex-col z-20 shadow-lg">
    <!-- Header & Search -->
    <div class="p-4 border-b border-border flex flex-col gap-3">
        <h2 class="text-xs font-bold uppercase tracking-widest text-text-subtle">
            Node Library
        </h2>
        
        <div class="relative">
            <Search size={14} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
            <input 
                type="text" 
                placeholder="Search nodes... (/)" 
                bind:value={searchQuery}
                class="w-full bg-bg-input border border-border rounded-md py-1.5 pl-8 pr-3 text-sm text-text placeholder:text-text-muted focus:outline-none focus:border-brand transition-colors"
                onkeydown={(e) => {
                    // Prevent canvas shortcuts when typing in search
                    e.stopPropagation();
                }}
            />
        </div>
    </div>

    <!-- Body / List -->
    {#if loading}
        <div class="p-4 text-xs text-text-muted animate-pulse flex items-center gap-2">
            <div class="w-3 h-3 border-2 border-brand border-t-transparent rounded-full animate-spin"></div>
            Loading registry...
        </div>
    {:else if error}
        <div class="p-4">
            <div class="text-xs text-status-error border border-status-error/30 bg-status-error/10 p-3 rounded-md">
                {error}
            </div>
        </div>
    {:else}
        <div class="flex-1 overflow-y-auto p-2 space-y-4 scrollbar-hide">
            {#each Object.entries(groupedTemplates).sort() as [category, nodes]}
                {@const isCollapsed = collapsedCategories.has(category)}
                {@const catColors = getCategoryColors(category)}
                {@const CatIcon = getNodeIcon(category)}
                
                <div class="flex flex-col gap-1">
                    <!-- Category Header -->
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div 
                        class="flex items-center gap-2 px-2 py-1.5 hover:bg-bg-hover rounded-md cursor-pointer select-none transition-colors group"
                        onclick={() => toggleCategory(category)}
                    >
                        <div class="w-4 flex justify-center text-text-subtle group-hover:text-text transition-colors">
                            {#if isCollapsed}
                                <ChevronRight size={14} />
                            {:else}
                                <ChevronDown size={14} />
                            {/if}
                        </div>
                        <div class="{catColors.accent} w-5 h-5 rounded flex items-center justify-center text-white/90">
                            <CatIcon size={12} strokeWidth={2.5} />
                        </div>
                        <span class="text-xs font-semibold text-text">{category}</span>
                        <span class="ml-auto text-[10px] text-text-subtle font-mono">{nodes.length}</span>
                    </div>

                    <!-- Nodes in Category -->
                    {#if !isCollapsed}
                        <div class="flex flex-col gap-0.5 pl-3 border-l border-border/50 ml-4 py-1">
                            {#each nodes as node}
                                {@const NodeIcon = getNodeIcon(category, node.name)}
                                <!-- svelte-ignore a11y_no_static_element_interactions -->
                                <div
                                    class="group/node flex items-center justify-between px-2 py-1.5 hover:bg-bg-hover rounded-md cursor-grab active:cursor-grabbing transition-colors"
                                    draggable="true"
                                    ondblclick={async () => {
                                        const randX = 400 + (Math.random() * 50 - 25);
                                        const randY = 300 + (Math.random() * 50 - 25);
                                        await backend.addNode(node.id, randX, randY);
                                        window.dispatchEvent(new CustomEvent("ferroflux:graph-change"));
                                    }}
                                    ondragstart={(e) => {
                                        e.dataTransfer?.setData("text/plain", node.id);
                                        if (e.dataTransfer) e.dataTransfer.effectAllowed = "copy";
                                        window.dispatchEvent(new CustomEvent("ferroflux:drag-start"));
                                    }}
                                    ondragend={() => {
                                        window.dispatchEvent(new CustomEvent("ferroflux:drag-end"));
                                    }}
                                >
                                    <div class="flex items-center gap-2">
                                        <div class="text-text-subtle group-hover/node:{catColors.headerText} transition-colors">
                                            <NodeIcon size={12} />
                                        </div>
                                        <span class="text-sm font-medium text-text-muted group-hover/node:text-text transition-colors">
                                            {node.name}
                                        </span>
                                    </div>
                                    <span class="text-[9px] text-text-subtle font-mono opacity-0 group-hover/node:opacity-100 transition-opacity">
                                        {node.node_type || "Node"}
                                    </span>
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/each}

            {#if Object.keys(groupedTemplates).length === 0}
                <div class="p-8 flex flex-col items-center justify-center text-center gap-2 text-text-muted h-32">
                    <Search size={24} class="opacity-30" />
                    <span class="text-xs">No nodes found for "{searchQuery}"</span>
                </div>
            {/if}
        </div>
    {/if}
</div>
