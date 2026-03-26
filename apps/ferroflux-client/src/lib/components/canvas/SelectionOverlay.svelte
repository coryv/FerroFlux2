<script lang="ts">
    import { CanvasState } from "$lib/logic/canvasState.svelte";

    let { state }: { state: CanvasState } = $props();

    function getRect() {
        if (!state.selectionStart || !state.selectionCurrent) return null;
        const x = Math.min(state.selectionStart.x, state.selectionCurrent.x);
        const y = Math.min(state.selectionStart.y, state.selectionCurrent.y);
        const w = Math.abs(state.selectionCurrent.x - state.selectionStart.x);
        const h = Math.abs(state.selectionCurrent.y - state.selectionStart.y);
        return { x, y, w, h };
    }
</script>

{#if state.selectionStart && state.selectionCurrent}
    {@const rect = getRect()}
    {#if rect}
        <div
            class="absolute border border-blue-500 bg-blue-500/20 pointer-events-none"
            style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px;"
        ></div>
    {/if}
{/if}
