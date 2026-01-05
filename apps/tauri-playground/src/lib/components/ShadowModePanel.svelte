<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    let { nodeId } = $props<{ nodeId: string }>();

    let runState = $state<"idle" | "running" | "success" | "error">("idle");
    let simulationResult = $state<any>(null);
    let showMockEditor = $state(false);
    let mockConfigJson = $state("{}");

    async function runSimulation() {
        if (!nodeId) return;
        runState = "running";
        simulationResult = null;
        try {
            let mocks = {};
            try {
                mocks = JSON.parse(mockConfigJson);
            } catch (e) {
                console.error("Invalid mock JSON");
            }

            let payload = {};

            const res = await invoke("simulate_node", {
                nodeId: nodeId,
                payload: payload,
                mocks: mocks,
            });
            simulationResult = res;
            runState = "success";
        } catch (e) {
            console.error("Simulation failed", e);
            simulationResult = { error: String(e) };
            runState = "error";
        }
    }
</script>

<div class="shadow-mode-panel">
    <div class="toolbar-actions">
        <button
            class="btn-icon"
            title="Toggle Mock Config"
            onclick={() => (showMockEditor = !showMockEditor)}
            class:active={showMockEditor}
        >
            <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline
                    points="7 10 12 15 17 10"
                /><line x1="12" y1="15" x2="12" y2="3" /></svg
            >
        </button>
        <button
            class="btn-icon btn-run"
            title="Run Node (Shadow Mode)"
            onclick={runSimulation}
            disabled={runState === "running"}
        >
            {#if runState === "running"}
                <span class="spinner"></span>
            {:else}
                <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><polygon points="5 3 19 12 5 21 5 3" /></svg
                >
            {/if}
        </button>
    </div>

    {#if showMockEditor}
        <div class="mock-editor-area">
            <label>
                Mock Config (JSON)
                <textarea
                    bind:value={mockConfigJson}
                    rows="5"
                    placeholder="JSON Config..."
                ></textarea>
            </label>
        </div>
    {/if}

    {#if simulationResult}
        <div class="result-area" class:error={runState === "error"}>
            <div class="result-header">
                <h4>Last Run</h4>
                <button
                    class="btn-icon small"
                    onclick={() => (simulationResult = null)}>×</button
                >
            </div>
            <pre>{JSON.stringify(simulationResult, null, 2)}</pre>
        </div>
    {/if}
</div>

<style>
    .shadow-mode-panel {
        display: flex;
        flex-direction: column;
    }
    .toolbar-actions {
        display: flex;
        gap: 8px;
        justify-content: flex-end;
    }
    .btn-icon {
        background: transparent;
        border: 1px solid var(--border-color);
        color: var(--text-secondary);
        border-radius: 4px;
        padding: 4px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
        width: 28px;
        height: 28px;
    }
    .btn-icon:hover {
        background: var(--bg-secondary);
        color: var(--text-primary);
    }
    .btn-icon.active {
        background: rgba(96, 165, 250, 0.2);
        color: var(--accent-color);
        border-color: var(--accent-color);
    }
    .btn-run {
        color: #4ade80;
        border-color: rgba(74, 222, 128, 0.2);
    }
    .btn-run:hover {
        background: rgba(74, 222, 128, 0.1);
    }
    .spinner {
        width: 14px;
        height: 14px;
        border: 2px solid rgba(255, 255, 255, 0.3);
        border-radius: 50%;
        border-top-color: #fff;
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
    .mock-editor-area {
        padding: 12px 0;
        margin-top: 8px;
    }
    label {
        font-size: 12px;
        color: var(--text-secondary);
        display: block;
        margin-bottom: 4px;
    }
    textarea {
        width: 100%;
        background: var(--bg-secondary);
        border: 1px solid var(--border-color);
        color: var(--text-primary);
        padding: 8px;
        border-radius: 4px;
        font-family: monospace;
        font-size: 12px;
        resize: vertical;
        box-sizing: border-box;
    }
    textarea:focus {
        outline: none;
        border-color: var(--accent-color);
    }
    .result-area {
        padding: 12px;
        background: var(--bg-primary);
        border: 1px solid var(--border-color);
        border-radius: 6px;
        margin-top: 8px;
        max-height: 200px;
        overflow-y: auto;
    }
    .result-area.error {
        background: rgba(239, 68, 68, 0.1);
        border-color: rgba(239, 68, 68, 0.2);
    }
    .result-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
    }
    .result-header h4 {
        margin: 0;
        font-size: 12px;
        color: var(--text-secondary);
    }
    .result-area pre {
        margin: 0;
        font-family: monospace;
        font-size: 10px;
        white-space: pre-wrap;
        color: var(--text-primary);
    }
    .btn-icon.small {
        width: 20px;
        height: 20px;
        font-size: 14px;
        padding: 0;
        border: none;
    }
</style>
