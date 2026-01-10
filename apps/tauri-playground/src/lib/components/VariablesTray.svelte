<script lang="ts">
    import type { SerializableNode } from "$lib/types";

    let { nodes = [], visible = false } = $props<{
        nodes: SerializableNode[];
        visible?: boolean;
    }>();

    let variables = $derived.by(() => {
        const vars = [
            { label: "Workflow ID", value: "{{workflow.id}}" },
            { label: "Execution ID", value: "{{system.execution_id}}" },
            { label: "Timestamp", value: "{{system.timestamp}}" },
            { label: "Webhook Base URL", value: "{{system.webhook_base}}" },
        ];

        for (const node of nodes) {
            // Check mappings
            const mappings = node.data.settings?.output_mappings || {};
            for (const varName of Object.values(mappings)) {
                if (typeof varName === "string" && varName) {
                    vars.push({ label: varName, value: `{{${varName}}}` });
                }
            }

            // Default step output using UUID
            // Use Name for display but UUID for reference to be robust
            vars.push({
                label: `${node.data.name || "Node"} Output`,
                value: `{{steps.${node.uuid}.output}}`,
            });
        }
        return vars;
    });

    function onDragStart(e: DragEvent, value: string) {
        if (!e.dataTransfer) return;
        e.dataTransfer.setData("text/plain", value);
        e.dataTransfer.effectAllowed = "copy";
    }
</script>

<div class="variables-tray" class:visible>
    <h4>Variables</h4>
    <div class="var-list">
        {#each variables as v}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="var-chip"
                draggable="true"
                ondragstart={(e) => onDragStart(e, v.value)}
                title="Drag to insert"
            >
                <div class="var-label">{v.label}</div>
                <div class="var-code">{v.value}</div>
            </div>
        {/each}
    </div>
</div>

<style>
    .variables-tray {
        position: absolute;
        top: auto;
        bottom: 0;
        right: 100%; /* Attach to left side of parent */
        margin-right: 12px;
        width: 200px;
        background: var(--panel-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 1px solid var(--border-color);
        border-right: none; /* Cohesive attachment */
        border-radius: 12px 0 0 12px;
        padding: 16px;
        box-shadow: 0 8px 32px var(--shadow-color);
        display: flex;
        flex-direction: column;
        gap: 12px;
        max-height: 60vh;
        overflow-y: auto;

        /* Animation */
        z-index: -1;
        transform: translateX(50%) scale(0.95);
        opacity: 0;
        pointer-events: none;
        transition:
            transform 0.4s cubic-bezier(0.16, 1, 0.3, 1),
            opacity 0.3s ease;
    }
    .variables-tray.visible {
        transform: translateX(0) scale(1);
        opacity: 1;
        pointer-events: auto;
    }

    h4 {
        margin: 0;
        font-size: 11px;
        text-transform: uppercase;
        color: var(--text-secondary);
        letter-spacing: 0.05em;
    }

    .var-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .var-chip {
        background: var(--bg-secondary);
        border: 1px solid var(--border-color);
        border-radius: 6px;
        padding: 8px;
        cursor: grab;
        transition:
            background 0.2s,
            border-color 0.2s;
    }
    .var-chip:hover {
        background: var(--bg-primary);
        border-color: var(--accent-color);
    }
    .var-chip:active {
        cursor: grabbing;
    }

    .var-label {
        font-size: 11px;
        color: var(--text-primary);
        font-weight: 500;
        margin-bottom: 2px;
    }
    .var-code {
        font-size: 10px;
        color: var(--accent-color);
        font-family: monospace;
        opacity: 0.8;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
