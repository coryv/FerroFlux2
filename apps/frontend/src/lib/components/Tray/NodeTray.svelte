<script lang="ts">
    import { onMount } from "svelte";
    import { useSdk } from "../../context/sdk.svelte";
    import type { NodeTemplate } from "../../api/adapter";

    const sdk = useSdk();
    let templates: NodeTemplate[] = $state([]);

    onMount(async () => {
        try {
            templates = await sdk.getNodeTemplates();
        } catch (e) {
            console.error("Failed to load templates:", e);
        }
    });

    function onDragStart(e: DragEvent, template: NodeTemplate) {
        if (e.dataTransfer) {
            e.dataTransfer.setData("application/ferroflux-node", template.id);
            e.dataTransfer.effectAllowed = "copy";
        }
    }
</script>

<div class="node-tray">
    <div class="tray-header">Node Library</div>

    <div class="tray-content">
        {#each templates as template}
            <div
                class="tray-item"
                draggable="true"
                ondragstart={(e) => onDragStart(e, template)}
                title={template.description}
                role="listitem"
            >
                <div class="category-tag">{template.category}</div>
                <div class="name">{template.name}</div>
            </div>
        {/each}
    </div>
</div>

<style>
    .node-tray {
        width: 250px;
        background: var(--panel-bg);
        border-right: 1px solid var(--border-color);
        display: flex;
        flex-direction: column;
        z-index: 100;
        backdrop-filter: blur(10px);
    }

    .tray-header {
        padding: 16px;
        font-weight: 600;
        border-bottom: 1px solid var(--border-color);
        text-transform: uppercase;
        letter-spacing: 1px;
        font-size: 0.8rem;
        color: var(--text-secondary);
    }

    .tray-content {
        flex: 1;
        overflow-y: auto;
        padding: 10px;
    }

    .tray-item {
        background: var(--bg-secondary);
        border: 1px solid var(--border-color);
        border-radius: 4px;
        padding: 10px;
        margin-bottom: 8px;
        cursor: grab;
        transition: all 0.2s;
    }

    .tray-item:hover {
        border-color: var(--accent-color);
        transform: translateY(-1px);
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    }

    .category-tag {
        font-size: 0.7em;
        text-transform: uppercase;
        color: var(--text-secondary);
        margin-bottom: 4px;
    }

    .name {
        font-weight: 500;
    }
</style>
