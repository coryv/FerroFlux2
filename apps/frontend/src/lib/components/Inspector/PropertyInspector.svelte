<script lang="ts">
    import { useGraph } from "../../stores/graph.svelte";
    import { useSdk } from "../../context/sdk.svelte";

    const graph = useGraph();
    const sdk = useSdk();

    // Derived state for the active node
    let selectedNodeId = $derived(
        graph.selectedNodes.size === 1
            ? Array.from(graph.selectedNodes)[0]
            : null,
    );

    let selectedNode = $derived(
        selectedNodeId ? graph.nodes[selectedNodeId] : null,
    );

    async function updateSetting(key: string, value: any) {
        if (!selectedNode || !selectedNodeId) return;

        // Optimistic update
        // We need to clone settings to trigger reactivity firmly if deeply nested,
        // though Svelte 5 fine-grained reactivity handles it if we mutate the state proxy.
        // graph.nodes is a proxy.
        graph.nodes[selectedNodeId].data.settings[key] = value;

        try {
            // Create a new object for the backend update
            const newSettings = {
                ...graph.nodes[selectedNodeId].data.settings,
            };
            await sdk.updateNodeSettings(selectedNodeId, newSettings);
        } catch (e) {
            console.error("Failed to update setting:", e);
        }
    }
</script>

<div class="property-inspector">
    <div class="inspector-header">Properties</div>

    <div class="inspector-content">
        {#if selectedNode}
            <div class="section">
                <div class="label">Name</div>
                <div class="value">{selectedNode.data.name}</div>
            </div>

            <div class="section">
                <div class="label">ID</div>
                <div class="value mono">{selectedNode.id}</div>
            </div>

            <div class="section-divider"></div>

            <div class="settings-list">
                {#each Object.entries(selectedNode.data.settings) as [key, value]}
                    <div class="setting-item">
                        <label for={key}>{key}</label>
                        <!-- Basic Input handling based on type -->
                        {#if typeof value === "boolean"}
                            <input
                                id={key}
                                type="checkbox"
                                checked={value}
                                onchange={(e) =>
                                    updateSetting(key, e.currentTarget.checked)}
                            />
                        {:else if typeof value === "number"}
                            <input
                                id={key}
                                type="number"
                                {value}
                                oninput={(e) =>
                                    updateSetting(
                                        key,
                                        parseFloat(e.currentTarget.value),
                                    )}
                            />
                        {:else}
                            <input
                                id={key}
                                type="text"
                                {value}
                                oninput={(e) =>
                                    updateSetting(key, e.currentTarget.value)}
                            />
                        {/if}
                    </div>
                {/each}
                {#if Object.keys(selectedNode.data.settings).length === 0}
                    <div class="empty-state">No settings available</div>
                {/if}
            </div>
        {:else}
            <div class="empty-state">Select a node to view properties</div>
        {/if}
    </div>
</div>

<style>
    .property-inspector {
        width: 300px;
        background: var(--panel-bg);
        border-left: 1px solid var(--border-color);
        display: flex;
        flex-direction: column;
        z-index: 100;
        backdrop-filter: blur(10px);
    }

    .inspector-header {
        padding: 16px;
        font-weight: 600;
        border-bottom: 1px solid var(--border-color);
        text-transform: uppercase;
        letter-spacing: 1px;
        font-size: 0.8rem;
        color: var(--text-secondary);
    }

    .inspector-content {
        padding: 16px;
        flex: 1;
        overflow-y: auto;
    }

    .section {
        margin-bottom: 12px;
    }

    .label {
        font-size: 0.75rem;
        color: var(--text-secondary);
        margin-bottom: 4px;
        text-transform: uppercase;
    }

    .value {
        font-size: 0.9rem;
    }

    .mono {
        font-family: monospace;
        font-size: 0.8rem;
        color: var(--accent-color);
    }

    .section-divider {
        height: 1px;
        background: var(--border-color);
        margin: 16px 0;
    }

    .setting-item {
        margin-bottom: 16px;
    }

    .setting-item label {
        display: block;
        font-size: 0.8rem;
        margin-bottom: 6px;
        color: var(--text-primary);
    }

    .setting-item input[type="text"],
    .setting-item input[type="number"] {
        width: 100%;
        background: var(--bg-primary);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        padding: 8px;
        border-radius: 4px;
        font-family: inherit;
    }

    .setting-item input:focus {
        outline: none;
        border-color: var(--accent-color);
    }

    .empty-state {
        color: var(--text-secondary);
        text-align: center;
        margin-top: 40px;
        font-style: italic;
    }
</style>
