<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { platform } from "@tauri-apps/plugin-os";
    import { onMount } from "svelte";

    const appWindow = getCurrentWindow();
    let osType = $state<string | null>(null);

    function minimize() {
        appWindow.minimize();
    }

    async function toggleMaximize() {
        const isMax = await appWindow.isMaximized();
        if (isMax) {
            appWindow.unmaximize();
        } else {
            appWindow.maximize();
        }
    }

    function close() {
        appWindow.close();
    }

    onMount(async () => {
        try {
            osType = await platform();
        } catch (e) {
            console.error("Failed to detect OS:", e);
        }
    });
</script>

<div class="header-layer" data-tauri-drag-region>
    <!-- Window Controls Pill -->
    <div class="controls-pill" class:macos={osType === "macos"}>
        {#if osType === "macos"}
            <div class="traffic-lights">
                <button onclick={close} class="mac-btn close" aria-label="Close"
                ></button>
                <button
                    onclick={minimize}
                    class="mac-btn minimize"
                    aria-label="Minimize"
                ></button>
                <button
                    onclick={toggleMaximize}
                    class="mac-btn maximize"
                    aria-label="Maximize"
                ></button>
            </div>
        {:else}
            <div class="win-controls">
                <button
                    onclick={minimize}
                    class="control-btn"
                    aria-label="Minimize"
                >
                    <svg width="10" height="1" viewBox="0 0 10 1"
                        ><path d="M0 0h10v1H0z" fill="currentColor" /></svg
                    >
                </button>
                <button
                    onclick={toggleMaximize}
                    class="control-btn"
                    aria-label="Maximize"
                >
                    <svg width="10" height="10" viewBox="0 0 10 10"
                        ><path
                            d="M0 0h10v10H0V0zm1 1v8h8V1H1z"
                            fill="currentColor"
                        /></svg
                    >
                </button>
                <button
                    onclick={close}
                    class="control-btn close"
                    aria-label="Close"
                >
                    <svg width="10" height="10" viewBox="0 0 10 10"
                        ><path
                            d="M1 0L0 1l4 4-4 4 1 1 4-4 4 4 1-1-4-4 4-4-1-1-4 4-4-4z"
                            fill="currentColor"
                        /></svg
                    >
                </button>
            </div>
        {/if}
    </div>

    <!-- Centered Title Pill -->
    <div class="title-pill">
        <img src="/favicon.png" alt="Logo" class="icon" />
        <span class="title">FerroFlux</span>
    </div>
</div>

<style>
    .header-layer {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        height: 56px; /* 12px padding top/bottom + 32px content */
        z-index: 9999;
        display: flex;
        justify-content: space-between;
        padding: 12px;
        pointer-events: auto; /* Enable clicks for drag region */
        /* Optional: Add very subtle gradient or keeping it transparent for existing aesthetic */
    }

    /* Make pills styles consistent */
    .controls-pill,
    .title-pill {
        /* pointer-events already auto, but parent is now auto too */
        pointer-events: auto;
        background: rgba(30, 30, 35, 0.6);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 999px; /* Capsule/Pill shape */
        display: flex;
        align-items: center;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }

    .controls-pill {
        padding: 6px 10px;
        height: 32px;
        box-sizing: border-box;
    }

    .traffic-lights {
        display: flex;
        gap: 8px;
    }

    .win-controls {
        display: flex;
        gap: 4px;
    }

    .title-pill {
        position: absolute;
        left: 50%;
        transform: translateX(-50%);
        top: 12px;
        padding: 6px 16px;
        height: 32px;
        gap: 8px;
        box-sizing: border-box;
    }

    .icon {
        width: 16px;
        height: 16px;
        opacity: 0.8;
    }

    .title {
        font-size: 13px;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.8);
        letter-spacing: 0.02em;
    }

    /* Buttons */
    .mac-btn {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        border: none;
        padding: 0;
        cursor: pointer;
        transition: transform 0.1s;
    }
    .mac-btn:active {
        transform: scale(0.9);
    }

    .mac-btn.close {
        background-color: #ff5f57;
        border: 1px solid #e0443e;
    }
    .mac-btn.minimize {
        background-color: #febc2e;
        border: 1px solid #dba524;
    }
    .mac-btn.maximize {
        background-color: #28c840;
        border: 1px solid #1aa52b;
    }

    /* Hover symbols for mac */
    .controls-pill:hover .mac-btn.close::before {
        content: "×";
        position: absolute;
        top: -2px;
        left: 3px;
        font-size: 10px;
        color: rgba(0, 0, 0, 0.6);
    }
    .controls-pill:hover .mac-btn.minimize::before {
        content: "−";
        position: absolute;
        top: -3px;
        left: 3px;
        font-size: 10px;
        color: rgba(0, 0, 0, 0.6);
    }
    .controls-pill:hover .mac-btn.maximize::before {
        content: "+";
        position: absolute;
        top: -2px;
        left: 3px;
        font-size: 10px;
        color: rgba(0, 0, 0, 0.6);
    }

    .control-btn {
        background: transparent;
        border: none;
        color: #ccc;
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        border-radius: 4px;
    }
    .control-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }
    .control-btn.close:hover {
        background: #c42b1c;
    }
</style>
