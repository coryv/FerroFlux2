<script lang="ts">
    import { getBackend } from "../context/backend.svelte";
    import { onMount } from "svelte";

    // Type definition for NodeMetadata (should mirror Rust)
    interface PortMetadata {
        name: string;
        data_type: string;
    }

    interface NodeMetadata {
        id: string;
        name: string;
        category: string;
        description?: string;
        inputs: PortMetadata[];
        outputs: PortMetadata[];
    }

    let templates: NodeMetadata[] = $state([]);
    let loading = $state(true);
    let error = $state<string | null>(null);

    const backend = getBackend();

    onMount(async () => {
        try {
            const data = await backend.getTemplates();
            // Group by category if needed, but flat list first
            templates = data;
        } catch (e: any) {
            error = e.message || "Failed to load templates";
            console.error(e);
        } finally {
            loading = false;
        }
    });
</script>

<div
    class="w-64 border-r border-white/20 bg-[#121212] h-full flex flex-col p-4"
>
    <h2
        class="text-sm font-bold uppercase tracking-wider text-neutral-400 mb-4"
    >
        Node Library
    </h2>

    {#if loading}
        <div class="text-xs text-neutral-500 animate-pulse">
            Scanning registry...
        </div>
    {:else if error}
        <div
            class="text-xs text-red-500 border border-red-900 bg-red-900/10 p-2 rounded"
        >
            ERROR: {error}
        </div>
    {:else}
        <div class="flex-1 overflow-y-auto pr-2 space-y-1">
            {#each templates as node}
                <div
                    class="group flex flex-col bg-white/5 hover:bg-white/10 p-2 rounded border border-transparent hover:border-white/20 cursor-move transition-all active:scale-95"
                    draggable="true"
                >
                    <div class="flex justify-between items-center">
                        <span class="text-sm font-medium text-neutral-200"
                            >{node.name}</span
                        >
                        <span class="text-[10px] text-neutral-500 font-mono"
                            >{node.id}</span
                        >
                    </div>
                    {#if node.category}
                        <div
                            class="text-[10px] text-neutral-600 uppercase mt-1"
                        >
                            {node.category}
                        </div>
                    {/if}
                </div>
            {/each}

            {#if templates.length === 0}
                <div class="text-xs text-neutral-600 italic">
                    No templates found.
                </div>
            {/if}
        </div>
    {/if}
</div>
