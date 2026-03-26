<script lang="ts">
    import { initBackendContext } from "$lib/context/backend.svelte";
    import NodeList from "$lib/components/NodeList.svelte";
    import InfiniteCanvas from "$lib/components/InfiniteCanvas.svelte";

    // Initialize backend context at root
    initBackendContext();
</script>

<div class="h-full w-full flex bg-[#0F0F0F] text-white overflow-hidden">
    <!-- Node Tray (Left Pane) -->
    <div class="w-64 border-r border-[#333] flex flex-col z-10 bg-[#0F0F0F]">
        <NodeList />
    </div>

    <!-- Canvas Area (Right Pane) -->
    <div class="flex-1 relative overflow-hidden">
        <InfiniteCanvas />
    </div>
</div>
