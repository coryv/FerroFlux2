<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    const nodeTemplates = [
        {
            type: "Trigger",
            name: "HTTP Request",
            icon: "M13 3v7h7l-9 11v-7H4l9-11z",
            id: "http_request",
        },
        {
            type: "Action",
            name: "Send Email",
            icon: "M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z",
            id: "send_email",
        },
        {
            type: "Logic",
            name: "Condition",
            icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z",
            id: "condition",
        },
        {
            type: "Database",
            name: "Query SQL",
            icon: "M12 2c4.418 0 8 1.119 8 2.5S16.418 7 12 7s-8-1.119-8-2.5S7.582 2 12 2zM4 6.74c0 1.381 3.582 2.5 8 2.5s8-1.119 8-2.5V9.4c0 1.381-3.582 2.5-8 2.5s-8-1.119-8-2.5V6.74zm0 2.66c0 1.381 3.582 2.5 8 2.5s8-1.119 8-2.5V12c0 1.381-3.582 2.5-8 2.5s-8-1.119-8-2.5V9.4z",
            id: "query_sql",
        },
    ];

    function onDragStart(e: DragEvent, template: (typeof nodeTemplates)[0]) {
        console.log("NodeTray: onDragStart", template.name);
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
    <div class="tray-items">
        {#each nodeTemplates as template}
            <div
                class="tray-item"
                draggable="true"
                role="listitem"
                ondragstart={(e) => onDragStart(e, template)}
                ondragend={(e) => {
                    invoke("log_js", {
                        msg:
                            "NodeTray: onDragEnd. DropEffect: " +
                            (e.dataTransfer?.dropEffect || "none"),
                    });
                }}
            >
                <div class="icon">
                    <svg viewBox="0 0 24 24" width="20" height="20">
                        <path fill="currentColor" d={template.icon} />
                    </svg>
                </div>
                <div class="info">
                    <span class="name">{template.name}</span>
                    <span class="type">{template.type}</span>
                </div>
            </div>
        {/each}
    </div>
</div>

<style>
    .node-tray {
        position: absolute;
        top: 24px;
        left: 24px;
        width: 180px;
        background: rgba(30, 30, 35, 0.85);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 16px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        z-index: 1000;
        user-select: none;
    }
    h3 {
        margin: 0 0 16px 0;
        font-size: 13px;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: #888;
        font-weight: 700;
    }
    .tray-items {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }
    .tray-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 10px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 8px;
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
    .icon {
        color: #60a5fa;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .info {
        display: flex;
        flex-direction: column;
    }
    .name {
        font-size: 12px;
        font-weight: 600;
        color: #eee;
    }
    .type {
        font-size: 10px;
        color: #666;
    }
</style>
