<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { GraphState } from "$lib/types";
    import Toolbar from "$lib/components/Toolbar.svelte";
    import Canvas from "$lib/components/Canvas.svelte";
    import TitleBar from "$lib/components/TitleBar.svelte";
    import NodeTray from "$lib/components/NodeTray.svelte";

    // State
    let graph = $state<GraphState>({ nodes: {}, edges: {}, draw_order: [] });
    let status = $state("Loading...");

    async function refreshGraph() {
        try {
            const rawGraph = await invoke("get_graph");
            console.log("Received Graph:", rawGraph);
            graph = rawGraph as GraphState;
            status =
                "Ready. Nodes: " +
                (graph.nodes ? Object.keys(graph.nodes).length : "0");
        } catch (e) {
            status = "Error: " + e;
        }
    }

    async function init() {
        try {
            status = "Initializing SDK...";
            await invoke("init_sdk");
            status = "Fetching Graph...";
            await refreshGraph();
        } catch (e) {
            status = "Init Error: " + JSON.stringify(e);
            console.error(e);
        }
    }



    async function deploy() {
        status = "Deploying...";
        try {
            await invoke("deploy");
            status = "Deployed & Ticked!";
        } catch (e) {
            status = "Deploy Error: " + e;
        }
    }

    onMount(init);
</script>

<svelte:window />

<main>
    <TitleBar />
    <div class="app-content">
        <Canvas {graph} {status} onDeploy={deploy} onRefresh={refreshGraph} />
        <NodeTray />
    </div>
</main>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        font-family: sans-serif;
        overflow: hidden;
        user-select: none;
        background: transparent; /* Allow window transparency */
        color: #eee;
    }

    /* Main Container (Visible Window) */
    main {
        position: relative;
        width: 100vw;
        height: 100vh;
        overflow: hidden;
        background: #111; /* Actual window background */
        border-radius: 12px; /* Rounded corners */
        box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.1); /* Subtle border */
    }

    .app-content {
        position: absolute;
        top: 0; /* Full height since header is floating */
        left: 0;
        right: 0;
        bottom: 0;
        overflow: hidden;
    }
</style>
