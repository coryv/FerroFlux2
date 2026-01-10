<script lang="ts">
    import type { NodeTemplate } from "$lib/types";

    let {
        template,
        settings,
        onUpdate,
        visible = false,
    } = $props<{
        template: NodeTemplate | null;
        settings: Record<string, any>;
        onUpdate: (key: string, value: any) => void;
        visible?: boolean;
    }>();

    function updateMapping(portName: string, variableName: string) {
        const mappings = { ...(settings.output_mappings || {}) };
        if (variableName.trim() === "") {
            delete mappings[portName];
        } else {
            mappings[portName] = variableName.trim();
        }
        onUpdate("output_mappings", mappings);
    }
</script>

<div class="output-mapper" class:visible>
    <h4>Output Mapping</h4>
    <div class="description">
        Assign node outputs to global variables for use in subsequent nodes.
    </div>

    <div class="mapper-list">
        {#if template && template.outputs}
            {#each template.outputs as out}
                {#if out.name !== "Success" && out.name !== "Error" && out.name !== "Exec"}
                    <!-- Filter out flow ports, strictly data ports? 
                          Actually "Success" implies flow. we only map data ports. 
                          But sometimes "Success" carries payload? 
                          Usually data outputs are specific named ports. -->
                    <div class="map-row">
                        <div class="source-field">
                            <span class="dot"></span>
                            {out.name}
                        </div>
                        <div class="arrow">→</div>
                        <input
                            type="text"
                            placeholder="Variable Name (e.g. my_var)"
                            value={settings.output_mappings?.[out.name] || ""}
                            onchange={(e) =>
                                updateMapping(
                                    out.name,
                                    (e.target as HTMLInputElement).value,
                                )}
                        />
                    </div>
                {/if}
            {/each}
        {:else}
            <div class="empty">No outputs available to map.</div>
        {/if}
    </div>
</div>

<style>
    .output-mapper {
        position: absolute;
        top: auto;
        bottom: 0;
        left: 100%; /* Attach to right side of parent */
        margin-left: 12px;
        width: 240px;
        background: var(--panel-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 1px solid var(--border-color);
        border-left: none; /* Cohesive attachment */
        border-radius: 0 12px 12px 0;
        padding: 16px;
        box-shadow: 0 8px 32px var(--shadow-color);
        display: flex;
        flex-direction: column;
        gap: 12px;
        color: var(--text-primary);
        max-height: 60vh;
        overflow-y: auto;

        /* Animation */
        z-index: -1;
        transform: translateX(-50%) scale(0.95);
        opacity: 0;
        pointer-events: none;
        transition:
            transform 0.4s cubic-bezier(0.16, 1, 0.3, 1),
            opacity 0.3s ease;
    }
    .output-mapper.visible {
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

    .description {
        font-size: 10px;
        color: var(--text-secondary);
        line-height: 1.3;
    }

    .mapper-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .map-row {
        display: flex;
        flex-direction: column;
        gap: 4px;
        background: var(--bg-primary);
        padding: 8px;
        border-radius: 6px;
        border: 1px solid var(--border-color);
    }

    .source-field {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 11px;
        font-weight: 600;
        color: var(--text-secondary);
    }
    .dot {
        width: 6px;
        height: 6px;
        background: var(--accent-color);
        border-radius: 50%;
    }

    .arrow {
        display: none; /* Implicit in layout */
    }

    input {
        background: var(--bg-secondary);
        border: 1px solid var(--border-color);
        border-radius: 4px;
        padding: 4px 8px;
        color: var(--accent-color);
        font-size: 11px;
        font-family: monospace;
    }
    input::placeholder {
        color: var(--text-secondary);
        font-family: sans-serif;
    }
    input:focus {
        outline: none;
        border-color: var(--accent-color);
    }

    .empty {
        font-size: 11px;
        color: var(--text-secondary);
        font-style: italic;
    }
</style>
