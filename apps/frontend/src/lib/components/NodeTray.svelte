<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { NodeTemplate } from "$lib/types";
    import {
        nodeRegistry,
        setTemplates,
    } from "$lib/stores/nodeRegistry.svelte";

    let categories: Record<string, NodeTemplate[]> = $state({});
    let selectedCategory = $state<string | null>(null);

    $effect(() => {
        const c: Record<string, NodeTemplate[]> = {};
        Object.values(nodeRegistry.templates).forEach((t) => {
            if (!c[t.category]) c[t.category] = [];
            c[t.category].push(t);
        });
        categories = c;
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
    async function reloadPalette() {
        try {
            await invoke("reload_definitions");
            const temps = await invoke<NodeTemplate[]>("get_node_templates");
            setTemplates(temps);
        } catch (e) {
            console.error("Failed to reload palette", e);
        }
    }
    let isCollapsed = $state(false);

    function toggleTray() {
        isCollapsed = !isCollapsed;
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="node-tray"
    class:collapsed={isCollapsed}
    onmousedown={(e) => e.stopPropagation()}
>
    <div class="tray-header">
        <div class="header-left">
            <button
                class="btn-toggle"
                onclick={toggleTray}
                title="Toggle Palette"
            >
                <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    <line x1="3" y1="12" x2="21" y2="12"></line>
                    <line x1="3" y1="6" x2="21" y2="6"></line>
                    <line x1="3" y1="18" x2="21" y2="18"></line>
                </svg>
            </button>
            <h3 class:hidden={isCollapsed}>Palette</h3>
        </div>

        <button
            class="btn-refresh"
            title="Reload Definitions"
            onclick={reloadPalette}
            class:hidden={isCollapsed}
        >
            <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M23 4v6h-6"></path>
                <path d="M1 20v-6h6"></path>
                <path
                    d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"
                ></path>
            </svg>
        </button>
    </div>

    <!-- Content only visible when expanded -->
    {#if !isCollapsed}
        <div class="tray-content">
            {#if !selectedCategory}
                <!-- Top Level: Category List -->
                <div class="view categories-view">
                    {#each Object.keys(categories) as category}
                        <button
                            class="nav-item"
                            onclick={() => (selectedCategory = category)}
                        >
                            <span class="label">{category}</span>
                            <span class="count"
                                >{categories[category].length}</span
                            >
                            <span class="chevron">›</span>
                        </button>
                    {/each}
                </div>
            {:else}
                <!-- Detail Level: Node List -->
                <div class="view nodes-view">
                    <button
                        class="btn-back"
                        onclick={() => (selectedCategory = null)}
                    >
                        <svg
                            width="12"
                            height="12"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="3"
                        >
                            <polyline points="15 18 9 12 15 6"></polyline>
                        </svg>
                        <span>{selectedCategory}</span>
                    </button>

                    <div class="node-list">
                        {#each categories[selectedCategory] as template}
                            <div
                                class="tray-item"
                                draggable="true"
                                role="listitem"
                                ondragstart={(e) => onDragStart(e, template)}
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
                </div>
            {/if}
        </div>
    {/if}
</div>

<style>
    .node-tray {
        position: absolute;
        top: 68px;
        left: 12px;
        bottom: 12px;
        width: 240px;
        background: rgba(30, 30, 35, 0.95);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        z-index: 1000;
        user-select: none;
        color: #eee;
        display: flex;
        flex-direction: column;
        transition:
            width 0.3s cubic-bezier(0.25, 1, 0.5, 1),
            padding 0.3s ease;
        overflow: hidden; /* Hide overflow for sliding views */
    }

    .node-tray.collapsed {
        width: 48px;
        padding: 16px 8px;
    }

    .tray-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px;
        padding-bottom: 8px;
    }

    .node-tray.collapsed .tray-header {
        justify-content: center;
        padding: 16px 0;
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    h3 {
        margin: 0;
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.15em;
        color: #666;
        font-weight: 700;
    }

    .btn-toggle,
    .btn-refresh {
        background: transparent;
        border: none;
        color: #888;
        cursor: pointer;
        padding: 4px;
        display: flex;
        border-radius: 4px;
    }

    .btn-toggle:hover,
    .btn-refresh:hover {
        color: #60a5fa;
        background: rgba(255, 255, 255, 0.05);
    }

    .tray-content {
        flex: 1;
        position: relative;
        overflow: hidden;
    }

    .view {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        padding: 0 12px 12px 12px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 4px;
        animation: slideIn 0.25s cubic-bezier(0.2, 0, 0.2, 1);
    }

    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateX(20px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    /* Categories View Styles */
    .nav-item {
        display: flex;
        align-items: center;
        padding: 10px 12px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid transparent;
        border-radius: 8px;
        color: #eee;
        font-size: 13px;
        cursor: pointer;
        text-align: left;
        transition: all 0.2s ease;
    }

    .nav-item:hover {
        background: rgba(255, 255, 255, 0.08);
        border-color: rgba(255, 255, 255, 0.1);
    }

    .nav-item .label {
        flex: 1;
        font-weight: 500;
    }

    .nav-item .count {
        font-size: 10px;
        color: #555;
        background: rgba(255, 255, 255, 0.05);
        padding: 2px 6px;
        border-radius: 10px;
        margin-right: 8px;
    }

    .nav-item .chevron {
        color: #444;
        font-size: 16px;
    }

    /* Nodes View Styles */
    .btn-back {
        display: flex;
        align-items: center;
        gap: 8px;
        background: transparent;
        border: none;
        color: #60a5fa;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        padding: 8px 0;
        margin-bottom: 8px;
    }

    .btn-back:hover {
        opacity: 0.8;
    }

    .node-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .tray-item {
        padding: 10px 12px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 8px;
        cursor: grab;
        transition: all 0.2s ease;
    }

    .tray-item:hover {
        background: rgba(255, 255, 255, 0.07);
        border-color: rgba(96, 165, 250, 0.3);
        transform: scale(1.02);
    }

    .info {
        display: flex;
        flex-direction: column;
    }
    .name {
        font-size: 13px;
        font-weight: 600;
        color: #eee;
    }
    .desc {
        font-size: 11px;
        color: #666;
        margin-top: 2px;
    }

    .hidden {
        display: none !important;
    }
</style>
