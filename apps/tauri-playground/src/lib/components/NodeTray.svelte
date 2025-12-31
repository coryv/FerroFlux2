<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { NodeTemplate } from "$lib/types";
    import {
        nodeRegistry,
        setTemplates,
    } from "$lib/stores/nodeRegistry.svelte";

    let categories: Record<string, NodeTemplate[]> = $state({});
    let openCategories: Record<string, boolean> = $state({});

    $effect(() => {
        const cats: Record<string, NodeTemplate[]> = {};
        Object.values(nodeRegistry.templates).forEach((t) => {
            if (!cats[t.category]) cats[t.category] = [];
            cats[t.category].push(t);
        });
        categories = cats;
        // Default open all if not already set
        if (Object.keys(openCategories).length === 0) {
            Object.keys(cats).forEach((k) => (openCategories[k] = true));
        }
    });

    onMount(async () => {
        let attempts = 0;
        while (attempts < 20) {
            try {
                const temps =
                    await invoke<NodeTemplate[]>("get_node_templates");
                setTemplates(temps);
                break;
            } catch (e) {
                const msg = String(e);
                if (msg.includes("Client not initialized")) {
                    // SDK not ready yet, wait and retry
                    await new Promise((r) => setTimeout(r, 250));
                    attempts++;
                } else {
                    console.error("Failed to fetch node templates", e);
                    break;
                }
            }
        }
    });

    function toggleCategory(cat: string) {
        openCategories[cat] = !openCategories[cat];
    }

    function onDragStart(e: DragEvent, template: NodeTemplate) {
        invoke("log_js", { msg: "NodeTray: onDragStart " + template.name });
        if (e.dataTransfer) {
            e.dataTransfer.setData(
                "application/ferroflux-node+json",
                JSON.stringify({ ...template, __ferroflux: true }),
            );
            e.dataTransfer.effectAllowed = "copy";
        }
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="node-tray" onmousedown={(e) => e.stopPropagation()}>
    <h3>Palette</h3>
    <div class="tray-content">
        {#each Object.entries(categories) as [category, items]}
            <div class="category">
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <div
                    class="category-header"
                    onclick={() => toggleCategory(category)}
                >
                    <span class="arrow" class:open={openCategories[category]}
                        >▶</span
                    >
                    {category}
                </div>
                {#if openCategories[category]}
                    <div class="category-items">
                        {#each items as template}
                            <div
                                class="tray-item"
                                draggable="true"
                                role="listitem"
                                ondragstart={(e) => onDragStart(e, template)}
                                ondragend={(e) => {
                                    invoke("log_js", {
                                        msg:
                                            "NodeTray: onDragEnd. DropEffect: " +
                                            (e.dataTransfer?.dropEffect ||
                                                "none"),
                                    });
                                }}
                            >
                                <div class="info">
                                    <span class="name">{template.name}</span>
                                    {#if template.description}
                                        <span class="desc"
                                            >{template.description}</span
                                        >
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {/each}
    </div>
</div>

<style>
    .node-tray {
        position: absolute;
        top: 24px;
        left: 24px;
        width: 220px;
        max-height: 80vh;
        overflow-y: auto;
        background: rgba(30, 30, 35, 0.95);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 16px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        z-index: 1000;
        user-select: none;
        color: #eee;
        display: flex;
        flex-direction: column;
    }
    h3 {
        margin: 0 0 16px 0;
        font-size: 13px;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: #888;
        font-weight: 700;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        padding-bottom: 8px;
    }
    .tray-content {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }
    .category-header {
        font-size: 11px;
        font-weight: 700;
        color: #aaa;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 6px;
        margin-bottom: 6px;
        text-transform: uppercase;
    }
    .category-header:hover {
        color: #fff;
    }
    .arrow {
        font-size: 8px;
        transition: transform 0.2s;
    }
    .arrow.open {
        transform: rotate(90deg);
    }
    .category-items {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-left: 8px;
        padding-left: 8px;
        border-left: 1px solid rgba(255, 255, 255, 0.05);
    }
    .tray-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px 10px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 6px;
        cursor: grab;
        transition: all 0.2s ease;
    }
    .tray-item:hover {
        background: rgba(255, 255, 255, 0.08);
        border-color: rgba(96, 165, 250, 0.3);
        transform: translateX(4px);
    }
    .tray-item:active {
        cursor: grabbing;
    }
    .info {
        display: flex;
        flex-direction: column;
    }
    .name {
        font-size: 12px;
        font-weight: 500;
        color: #eee;
    }
    .desc {
        font-size: 10px;
        color: #666;
        margin-top: 2px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 140px;
    }
</style>
