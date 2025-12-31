<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type { GraphState } from "$lib/types";
    import Toolbar from "$lib/components/Toolbar.svelte";
    import Canvas from "$lib/components/Canvas.svelte";
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

    async function addNode() {
        try {
            const x = 100 + Math.random() * 400;
            const y = 100 + Math.random() * 400;
            await invoke("add_node", { name: "New Node", x, y });
            await refreshGraph();
        } catch (e) {
            status = "Add Node Error: " + e;
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
    <Canvas {graph} {status} onDeploy={deploy} onRefresh={refreshGraph} />
    <NodeTray />
</main>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        font-family: sans-serif;
        overflow: hidden;
        user-select: none;
        background: #111;
        color: #eee;
    }
</style>
