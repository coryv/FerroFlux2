<script lang="ts">
    import "../app.css";
    import AppShell from "$lib/components/layout/AppShell.svelte";
    import { initBackendContext } from "$lib/context/backend.svelte";
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import { executionStore } from "$lib/logic/executionStore.svelte";
    import { setContext } from "svelte";

    let { children } = $props();

    // Initialize Global State
    const backend = initBackendContext();
    executionStore.init(backend);

    const canvasState = new CanvasState();
    setContext('canvas_state', canvasState);
</script>

<AppShell>
    {@render children()}
</AppShell>