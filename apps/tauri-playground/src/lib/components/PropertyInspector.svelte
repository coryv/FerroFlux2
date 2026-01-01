<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { NodeTemplate, SerializableNode } from "$lib/types";
    import { nodeRegistry } from "$lib/stores/nodeRegistry.svelte";
    import { workflowContext } from "$lib/stores/workflow.svelte";

    let { selectedNode, onRefresh } = $props<{
        selectedNode: SerializableNode | null;
        onRefresh: () => Promise<void>;
    }>();

    let template = $derived.by(() => {
        if (!selectedNode) return null;
        return nodeRegistry.templates[selectedNode.data.template_id] || null;
    });

    let settings = $state<Record<string, any>>({});

    // When selection changes, sync local settings
    $effect(() => {
        if (selectedNode) {
            settings = { ...selectedNode.data.settings };
        } else {
            settings = {};
        }
    });

    async function updateSetting(name: string, value: any) {
        if (!selectedNode) return;
        settings[name] = value;
        try {
            await invoke("update_node_settings", {
                nodeId: selectedNode.id,
                settings: settings,
            });
            await onRefresh();
        } catch (e) {
            console.error("Failed to update settings", e);
        }
    }

    function addRule() {
        const rules = settings.rules || [];
        rules.push({
            condition: "==",
            path: "",
            value: "",
            output: `Port ${rules.length + 1}`,
        });
        updateSetting("rules", rules);
    }

    function removeRule(index: number) {
        const rules = [...(settings.rules || [])];
        rules.splice(index, 1);
        updateSetting("rules", rules);
    }

    function updateRule(index: number, key: string, value: any) {
        const rules = [...(settings.rules || [])];
        rules[index] = { ...rules[index], [key]: value };
        updateSetting("rules", rules);
    }
    function evaluateCondition(expr: any): boolean {
        if (typeof expr !== "string") return true;
        const parts = expr.trim().split(/\s+/);
        if (parts.length === 3) {
            const [key, op, val] = parts;
            const currentVal = settings[key];
            const targetVal = (val || "").replace(/['"]/g, "");

            if (op === "==") return String(currentVal) === targetVal;
            if (op === "!=") return String(currentVal) !== targetVal;
        }
        return true;
    }

    function shouldShow(s: any): boolean {
        if (!s || !s.show_if || typeof s.show_if !== "string") return true;
        const showIf = s.show_if;

        // Handle logical OR "||"
        if (showIf.includes("||")) {
            const conditions = showIf.split("||");
            return conditions.some((c: string) => evaluateCondition(c.trim()));
        }

        return evaluateCondition(showIf);
    }

    function resolveTemplate(input: any): string {
        if (typeof input !== "string") return String(input);
        if (!input.includes("{{")) return input;

        const context: Record<string, any> = {
            system: {
                webhook_base: "http://localhost:8080",
            },
            workflow: {
                id: workflowContext.id,
            },
            settings: settings,
        };

        return input.replace(/\{\{\s*([^}]+?)\s*\}\}/g, (match, path) => {
            const parts = path.split(".");
            let current = context;
            for (const part of parts) {
                if (current && typeof current === "object") {
                    current = current[part];
                } else {
                    return match;
                }
            }
            return current !== undefined ? String(current) : match;
        });
    }

    function toggleMultiSelect(name: string, value: any) {
        const current = settings[name] || [];
        const index = current.indexOf(value);
        if (index > -1) {
            current.splice(index, 1);
        } else {
            current.push(value);
        }
        updateSetting(name, [...current]);
    }
    async function copyToClipboard(text: string) {
        try {
            await navigator.clipboard.writeText(text);
            // Optional: Show a brief "Copied!" toast or feedback
        } catch (err) {
            console.error("Failed to copy!", err);
        }
    }
</script>

<div
    class="property-inspector"
    class:visible={!!selectedNode}
    role="presentation"
    onmousedown={(e) => e.stopPropagation()}
>
    {#if selectedNode}
        {#if template}
            <div class="header">
                <h3>{template.name}</h3>
                <span class="type-id">{template.id}</span>
            </div>

            <div class="scroll-area">
                {#if template.description}
                    <div class="description">{template.description}</div>
                {/if}

                <div class="settings-group">
                    <h4>Settings</h4>
                    {#each template.settings as s}
                        {#if shouldShow(s)}
                            <div class="setting-item">
                                <label for={s.name}>{s.label}</label>

                                {#if s.read_only}
                                    {@const resolved = resolveTemplate(
                                        settings[s.name] || s.placeholder,
                                    )}
                                    <div class="read-only-container">
                                        <div class="read-only-val">
                                            {resolved || "---"}
                                        </div>
                                        <button
                                            class="btn-copy"
                                            title="Copy to Clipboard"
                                            onclick={() =>
                                                copyToClipboard(resolved)}
                                        >
                                            <svg
                                                width="12"
                                                height="12"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                ><rect
                                                    x="9"
                                                    y="9"
                                                    width="13"
                                                    height="13"
                                                    rx="2"
                                                    ry="2"
                                                /><path
                                                    d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
                                                /></svg
                                            >
                                        </button>
                                    </div>
                                {:else if s.type === "string" || s.type === "url" || s.type === "email"}
                                    <input
                                        id={s.name}
                                        type={s.type === "string"
                                            ? "text"
                                            : s.type}
                                        value={settings[s.name] || ""}
                                        placeholder={s.placeholder || ""}
                                        onchange={(e) =>
                                            updateSetting(
                                                s.name,
                                                (e.target as HTMLInputElement)
                                                    .value,
                                            )}
                                    />
                                {:else if s.type === "textarea"}
                                    <textarea
                                        id={s.name}
                                        value={settings[s.name] || ""}
                                        placeholder={s.placeholder || ""}
                                        rows="3"
                                        onchange={(e) =>
                                            updateSetting(
                                                s.name,
                                                (
                                                    e.target as HTMLTextAreaElement
                                                ).value,
                                            )}
                                    ></textarea>
                                {:else if s.type === "number"}
                                    <input
                                        id={s.name}
                                        type="number"
                                        min={s.min}
                                        max={s.max}
                                        step={s.step}
                                        value={settings[s.name] || 0}
                                        onchange={(e) =>
                                            updateSetting(
                                                s.name,
                                                Number(
                                                    (
                                                        e.target as HTMLInputElement
                                                    ).value,
                                                ),
                                            )}
                                    />
                                {:else if s.type === "boolean"}
                                    <input
                                        id={s.name}
                                        type="checkbox"
                                        checked={!!settings[s.name]}
                                        onchange={(e) =>
                                            updateSetting(
                                                s.name,
                                                (e.target as HTMLInputElement)
                                                    .checked,
                                            )}
                                    />
                                {:else if s.type === "time"}
                                    <input
                                        id={s.name}
                                        type="time"
                                        value={settings[s.name] || ""}
                                        onchange={(e) =>
                                            updateSetting(
                                                s.name,
                                                (e.target as HTMLInputElement)
                                                    .value,
                                            )}
                                    />
                                {:else if s.type === "select" || s.type === "enum" || s.options}
                                    <select
                                        id={s.name}
                                        value={settings[s.name]}
                                        onchange={(e) =>
                                            updateSetting(
                                                s.name,
                                                (e.target as HTMLSelectElement)
                                                    .value,
                                            )}
                                    >
                                        {#each s.options || [] as opt}
                                            {@const val =
                                                typeof opt === "string"
                                                    ? opt
                                                    : opt.value}
                                            {@const lab =
                                                typeof opt === "string"
                                                    ? opt
                                                    : opt.label}
                                            <option value={val}>{lab}</option>
                                        {/each}
                                    </select>
                                {:else if s.type === "multi-select"}
                                    <div class="multi-select-group">
                                        {#each s.options || [] as opt}
                                            {@const val =
                                                typeof opt === "string"
                                                    ? opt
                                                    : opt.value}
                                            {@const lab =
                                                typeof opt === "string"
                                                    ? opt
                                                    : opt.label}
                                            <label class="checkbox-label">
                                                <input
                                                    type="checkbox"
                                                    checked={(
                                                        settings[s.name] || []
                                                    ).includes(val)}
                                                    onchange={() =>
                                                        toggleMultiSelect(
                                                            s.name,
                                                            val,
                                                        )}
                                                />
                                                {lab}
                                            </label>
                                        {/each}
                                    </div>
                                {:else if s.type === "list" && s.name === "rules"}
                                    <div class="rule-builder">
                                        {#each settings.rules || [] as rule, i}
                                            <div class="rule-item">
                                                <div class="rule-row">
                                                    <input
                                                        placeholder="Path (e.g. user.id)"
                                                        value={rule.path}
                                                        onchange={(e) =>
                                                            updateRule(
                                                                i,
                                                                "path",
                                                                (
                                                                    e.target as HTMLInputElement
                                                                ).value,
                                                            )}
                                                    />
                                                    <select
                                                        value={rule.condition}
                                                        onchange={(e) =>
                                                            updateRule(
                                                                i,
                                                                "condition",
                                                                (
                                                                    e.target as HTMLSelectElement
                                                                ).value,
                                                            )}
                                                    >
                                                        <option value="=="
                                                            >==</option
                                                        >
                                                        <option value="!="
                                                            >!=</option
                                                        >
                                                        <option value=">"
                                                            >&gt;</option
                                                        >
                                                        <option value="<"
                                                            >&lt;</option
                                                        >
                                                        <option value="contains"
                                                            >contains</option
                                                        >
                                                    </select>
                                                </div>
                                                <div class="rule-row">
                                                    <input
                                                        placeholder="Value"
                                                        value={rule.value}
                                                        onchange={(e) =>
                                                            updateRule(
                                                                i,
                                                                "value",
                                                                (
                                                                    e.target as HTMLInputElement
                                                                ).value,
                                                            )}
                                                    />
                                                    <input
                                                        placeholder="Output Port"
                                                        value={rule.output}
                                                        onchange={(e) =>
                                                            updateRule(
                                                                i,
                                                                "output",
                                                                (
                                                                    e.target as HTMLInputElement
                                                                ).value,
                                                            )}
                                                    />
                                                    <button
                                                        class="btn-remove"
                                                        onclick={() =>
                                                            removeRule(i)}
                                                    >
                                                        ×
                                                    </button>
                                                </div>
                                            </div>
                                        {/each}
                                        <button
                                            class="btn-add"
                                            onclick={addRule}
                                        >
                                            + Add Rule
                                        </button>
                                    </div>
                                {:else}
                                    <div class="unsupported">
                                        Unsupported type: {s.type}
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    {/each}
                </div>
            </div>
        {:else}
            <div class="loading">Loading template...</div>
        {/if}
    {/if}
</div>

<style>
    .property-inspector {
        position: absolute;
        top: 12px;
        right: 12px;
        bottom: 12px;
        width: 280px;
        /* max-height: 80vh; */
        background: rgba(30, 30, 35, 0.95);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 0;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        z-index: 1000;
        color: #eee;
        display: flex;
        flex-direction: column;

        /* Animation */
        transition:
            transform 0.4s cubic-bezier(0.16, 1, 0.3, 1),
            opacity 0.2s ease;
        transform: translateX(320px);
        opacity: 0;
        pointer-events: none;
    }
    .property-inspector.visible {
        transform: translateX(0);
        opacity: 1;
        pointer-events: auto;
    }
    .header {
        padding: 16px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }
    h3 {
        margin: 0;
        font-size: 16px;
        color: #fff;
    }
    .type-id {
        font-size: 10px;
        color: #666;
        font-family: monospace;
    }
    .scroll-area {
        padding: 16px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 20px;
    }
    .description {
        font-size: 11px;
        color: #aaa;
        line-height: 1.4;
    }
    h4 {
        margin: 0 0 12px 0;
        font-size: 12px;
        text-transform: uppercase;
        color: #888;
        letter-spacing: 0.05em;
    }
    .setting-item {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 16px;
    }
    label {
        font-size: 11px;
        font-weight: 600;
        color: #bbb;
    }
    input[type="text"],
    input[type="number"],
    input[type="time"],
    select,
    textarea {
        background: rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        padding: 6px 8px;
        color: #fff;
        font-size: 12px;
        width: 100%;
        box-sizing: border-box;
    }
    textarea {
        resize: vertical;
        font-family: inherit;
    }
    input:focus,
    textarea:focus,
    select:focus {
        outline: none;
        border-color: #60a5fa;
    }
    .multi-select-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
        background: rgba(0, 0, 0, 0.2);
        padding: 8px;
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }
    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 12px;
        color: #eee;
        cursor: pointer;
    }
    .checkbox-label input {
        width: auto;
    }
    .read-only-container {
        display: flex;
        gap: 8px;
        align-items: center;
        background: rgba(96, 165, 250, 0.1);
        border-radius: 4px;
        padding: 2px 8px;
    }
    .read-only-val {
        font-size: 11px;
        color: #60a5fa;
        word-break: break-all;
        font-family: monospace;
        flex: 1;
        padding: 6px 0;
    }
    .btn-copy {
        background: transparent;
        border: none;
        color: #60a5fa;
        cursor: pointer;
        padding: 4px;
        display: flex;
        align-items: center;
        opacity: 0.6;
        transition: opacity 0.2s;
    }
    .btn-copy:hover {
        opacity: 1;
    }
    .rule-builder {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }
    .rule-item {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 6px;
        padding: 8px;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .rule-row {
        display: flex;
        gap: 6px;
        align-items: center;
    }
    .btn-remove {
        background: transparent;
        border: none;
        color: #ff4444;
        cursor: pointer;
        font-size: 18px;
        padding: 0 4px;
    }
    .btn-add {
        background: rgba(96, 165, 250, 0.1);
        border: 1px solid rgba(96, 165, 250, 0.2);
        color: #60a5fa;
        padding: 6px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 11px;
        font-weight: 600;
    }
    .btn-add:hover {
        background: rgba(96, 165, 250, 0.2);
    }
    .loading {
        padding: 40px 20px;
        text-align: center;
        color: #666;
        font-size: 12px;
    }
</style>
