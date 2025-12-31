<script lang="ts">
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    let {
        currentStyle = "Cubic",
        canUndo = true,
        canRedo = true,
        status = "",
        onDeploy,
    } = $props<{
        currentStyle?: "Cubic" | "Linear" | "Orthogonal";
        canUndo?: boolean;
        canRedo?: boolean;
        status?: string;
        onDeploy: () => void;
    }>();

    function setStyle(style: "Cubic" | "Linear" | "Orthogonal") {
        dispatch("setStyle", style);
    }
</script>

<div class="toolbar">
    <div class="group">
        <button class="primary" onclick={onDeploy} title="Deploy">
            <svg viewBox="0 0 24 24" width="18" height="18"
                ><path fill="currentColor" d="M5 3v18l15-9L5 3z" /></svg
            >
        </button>
    </div>

    <div class="divider"></div>

    <div class="group">
        <button
            onclick={() => dispatch("undo")}
            disabled={!canUndo}
            title="Undo (Cmd+Z)"
        >
            <svg viewBox="0 0 24 24" width="18" height="18"
                ><path
                    fill="currentColor"
                    d="M12.5 8c-2.65 0-5.05.99-6.9 2.6L2 7v9h9l-3.62-3.62c1.39-1.16 3.16-1.88 5.12-1.88 3.54 0 6.55 2.31 7.6 5.5l2.37-.78C21.08 11.03 17.15 8 12.5 8z"
                /></svg
            >
        </button>
        <button
            onclick={() => dispatch("redo")}
            disabled={!canRedo}
            title="Redo (Cmd+Shift+Z)"
        >
            <svg viewBox="0 0 24 24" width="18" height="18"
                ><path
                    fill="currentColor"
                    d="M18.4 10.6C16.55 8.99 14.15 8 11.5 8c-4.65 0-8.58 3.03-9.96 7.22L3.9 16c1.05-3.19 4.05-5.5 7.6-5.5 1.95 0 3.73.72 5.12 1.88L13 16h9V7l-3.6 3.6z"
                /></svg
            >
        </button>
    </div>

    <div class="divider"></div>

    <div class="group">
        <button
            class:active={currentStyle === "Cubic"}
            onclick={() => setStyle("Cubic")}
            title="Cubic Bezier"
        >
            <svg viewBox="0 0 24 24" width="18" height="18"
                ><path
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    d="M3 19c4-12 14-12 18 0"
                /></svg
            >
        </button>
        <button
            class:active={currentStyle === "Linear"}
            onclick={() => setStyle("Linear")}
            title="Linear"
        >
            <svg viewBox="0 0 24 24" width="18" height="18"
                ><path
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    d="M3 19 L21 5"
                /></svg
            >
        </button>
        <button
            class:active={currentStyle === "Orthogonal"}
            onclick={() => setStyle("Orthogonal")}
            title="Orthogonal"
        >
            <svg viewBox="0 0 24 24" width="18" height="18"
                ><path
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    d="M3 19 H12 V5 H21"
                /></svg
            >
        </button>
    </div>

    {#if status}
        <div class="divider"></div>
        <div class="status">{status}</div>
    {/if}
</div>

<style>
    .toolbar {
        position: absolute;
        bottom: 24px;
        left: 50%;
        transform: translateX(-50%);
        background: rgba(30, 30, 35, 0.85);
        backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 6px;
        display: flex;
        align-items: center;
        gap: 8px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        z-index: 1000;
        user-select: none;
    }
    .group {
        display: flex;
        gap: 4px;
    }
    .divider {
        width: 1px;
        height: 24px;
        background: rgba(255, 255, 255, 0.1);
    }
    button {
        background: transparent;
        border: none;
        color: #aaa;
        padding: 8px;
        border-radius: 8px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    }
    button:hover:not(:disabled) {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }
    button:active:not(:disabled) {
        transform: scale(0.92);
        background: rgba(255, 255, 255, 0.15);
    }
    button:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }
    button.active {
        background: var(--edge-selected, #60a5fa);
        color: #fff;
        box-shadow: 0 0 12px rgba(96, 165, 250, 0.4);
    }
    button.primary {
        color: #60a5fa;
    }
    button.primary:hover {
        background: rgba(96, 165, 250, 0.15);
    }
    .status {
        font-size: 11px;
        color: #888;
        padding: 0 8px;
        white-space: nowrap;
        max-width: 200px;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
