<script lang="ts">
    import Edge from "./Edge.svelte";
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import { findRoute } from "$lib/logic/router";

    let { state }: { state: CanvasState } = $props();

    // Cache or Computed?
    // For now simple reactive

    function getEdgePath(p1: any, p2: any): string {
        // Add stubs (Output is Right, Input is Left)
        const STUB_LENGTH = 20;
        const p1Stub = { x: p1.x + STUB_LENGTH, y: p1.y };
        const p2Stub = { x: p2.x - STUB_LENGTH, y: p2.y };

        // Use router between stubs
        const points = findRoute(p1Stub, p2Stub, state.getObstacles());

        // Construct path: p1 -> p1Stub -> ... -> p2Stub -> p2
        let d = `M ${p1.x} ${p1.y} L ${p1Stub.x} ${p1Stub.y}`;

        if (points.length > 0) {
            // If points[0] is same as Stub, router might have included it or not.
            // Our router includes start/end.
            // So we just iterate points.
            for (const p of points) {
                d += ` L ${p.x} ${p.y}`;
            }
        }

        d += ` L ${p2Stub.x} ${p2Stub.y} L ${p2.x} ${p2.y}`;
        return d;
    }
</script>

<svg class="absolute inset-0 w-full h-full overflow-visible">
    {#each state.graph.connections as conn (conn.id)}
        {@const p1 = state.getPortPosition(conn.from)}
        {@const p2 = state.getPortPosition(conn.to)}
        <Edge
            id={conn.id}
            path={getEdgePath(p1, p2)}
            selected={state.selectedEdges.has(conn.id)}
            onselect={(e) => {
                e.stopPropagation();
                if (e.shiftKey) {
                    state.selectedEdges.add(conn.id);
                    // Force reactivity if Set doesn't trigger it automatically in Svelte 5 proxy
                    state.selectedEdges = new Set(state.selectedEdges);
                } else {
                    state.selectedEdges = new Set([conn.id]);
                    // Only clear nodes if we want exclusive selection logic
                    if (!e.metaKey && !e.ctrlKey) {
                        state.selectedNodes = new Set();
                    }
                }
            }}
        />
    {/each}
    {#if state.dragEdgeStart && state.dragEdgeCurrent}
        {@const p1 = state.getPortPosition(state.dragEdgeStart)}
        {#if p1}
            <!-- Inline simple path for drag preview -->
            {@const d = `M ${p1.x} ${p1.y} L ${state.dragEdgeCurrent.x} ${state.dragEdgeCurrent.y}`}
            <!-- Or calculate bezier to mouse -->
            {@const dist = Math.abs(p1.x - state.dragEdgeCurrent.x) * 0.5}
            {@const dBez = `M ${p1.x} ${p1.y} C ${p1.x + dist} ${p1.y}, ${state.dragEdgeCurrent.x - dist} ${state.dragEdgeCurrent.y}, ${state.dragEdgeCurrent.x} ${state.dragEdgeCurrent.y}`}
            <path
                d={dBez}
                stroke="#fff"
                stroke-width="2"
                stroke-dasharray="5,5"
                fill="none"
                class="pointer-events-none"
            />
        {/if}
    {/if}
</svg>
