<script lang="ts">
    import { initBackendContext } from "$lib/context/backend.svelte";
    import TopBar from "$lib/components/layout/TopBar.svelte";
    import Sidebar from "$lib/components/layout/Sidebar.svelte";
    import NodeList from "$lib/components/NodeList.svelte";
    import InfiniteCanvas from "$lib/components/InfiniteCanvas.svelte";

    // Initialize backend context at root
    initBackendContext();
</script>

<div class="h-screen w-screen flex flex-col bg-bg text-text overflow-hidden font-sans">
    <TopBar />
    
    <div class="flex flex-1 overflow-hidden relative">
        <Sidebar />
        <NodeList />
        
        <!-- Canvas Area (Right Pane) -->
        <main class="flex-1 relative overflow-hidden bg-bg z-0 focus:outline-none">
            <InfiniteCanvas />
        </main>
    </div>
</div>
