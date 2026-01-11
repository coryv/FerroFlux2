<script lang="ts">
    import { onMount } from "svelte";
    import { initSdkContext } from "$lib/context/sdk.svelte";
    import { initGraphState } from "$lib/stores/graph.svelte";

    import NodeTray from "$lib/components/Tray/NodeTray.svelte";
    import InfiniteCanvas from "$lib/components/Canvas/InfiniteCanvas.svelte";
    import PropertyInspector from "$lib/components/Inspector/PropertyInspector.svelte";

    // Initialize Global Contexts (Order Matters)
    const sdk = initSdkContext();
    const graph = initGraphState();

    onMount(async () => {
        await sdk.initSdk();
        // graph.loadGraph() is called in constructor of GraphState
    });
</script>

<div class="app-container">
    <NodeTray />

    <div class="canvas-area">
        <InfiniteCanvas />
    </div>

    <PropertyInspector />
</div>

<style>
    .app-container {
        display: flex;
        width: 100vw;
        height: 100vh;
        background: var(--bg-primary);
        color: var(--text-primary);
    }

    .canvas-area {
        flex-grow: 1;
        position: relative;
        z-index: 1;
    }
</style>
