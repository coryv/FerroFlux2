<script lang="ts">
    import { initBackendContext } from "$lib/context/backend.svelte";
    import NodeList from "$lib/components/NodeList.svelte";

    // Initialize backend context at root
    initBackendContext();
</script>

<div class="h-screen w-screen flex bg-[#0F0F0F] text-white overflow-hidden">
    <!-- Node Tray -->
    <NodeList />

    <!-- Canvas Area -->
    <div class="flex-1 relative bg-grid-pattern">
        <!-- Canvas Placeholder -->
        <div
            class="absolute inset-0 flex items-center justify-center pointer-events-none"
        >
            <div class="flex flex-col items-center gap-4 opacity-20">
                <h1 class="text-4xl font-bold font-mono tracking-tight">
                    FerroFlux // Client
                </h1>
                <p class="font-mono">Infinite Canvas Ready</p>
            </div>
        </div>
    </div>
</div>

<style>
    .bg-grid-pattern {
        background-size: 40px 40px;
        background-image: linear-gradient(
                to right,
                rgba(255, 255, 255, 0.05) 1px,
                transparent 1px
            ),
            linear-gradient(
                to bottom,
                rgba(255, 255, 255, 0.05) 1px,
                transparent 1px
            );
    }
</style>
