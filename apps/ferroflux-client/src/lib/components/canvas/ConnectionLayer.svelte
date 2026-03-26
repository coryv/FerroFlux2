<script lang="ts">
    import Edge from "./Edge.svelte";
    import { CanvasState } from "$lib/logic/canvasState.svelte";
    import { getDataTypeColor } from "$lib/utils/nodeColors";

    let { state }: { state: CanvasState } = $props();

    function getBezierPath(p1: any, p2: any): string {
        const dx = Math.abs(p2.x - p1.x) * 0.5;
        // Make sure it goes outward from output (right) and inward to input (left)
        // Adjust control points based on distance
        const tension = Math.max(dx, 50);
        return `M ${p1.x} ${p1.y} C ${p1.x + tension} ${p1.y}, ${p2.x - tension} ${p2.y}, ${p2.x} ${p2.y}`;
    }

    function getConnectionColor(conn: any): string {
        try {
            const port = state.graph.ports[conn.from];
            if (!port) return "#555";
            return getDataTypeColor(port.data_type);
        } catch {
            return "#555";
        }
    }
</script>

<!-- The svg wrapper needs pointer-events-none so we can click nodes underneath,
     but the individual <path> elements inside Edge have pointer-events-auto. -->
<svg class="absolute inset-0 w-full h-full overflow-visible z-0 pointer-events-none">
    {#each state.graph.connections as conn (conn.id)}
        {@const p1 = state.getPortPosition(conn.from)}
        {@const p2 = state.getPortPosition(conn.to)}
        {#if p1 && p2}
            <Edge
                id={conn.id}
                path={getBezierPath(p1, p2)}
                color={getConnectionColor(conn)}
                selected={state.selectedEdges.has(conn.id)}
                animated={false /* Later derived from execution state */}
                onselect={(e) => {
                    e.stopPropagation();
                    if (e.shiftKey) {
                        state.selectedEdges.add(conn.id);
                        state.selectedEdges = new Set(state.selectedEdges);
                    } else {
                        state.selectedEdges = new Set([conn.id]);
                        if (!e.metaKey && !e.ctrlKey) {
                            state.selectedNodes = new Set();
                        }
                    }
                }}
            />
        {/if}
    {/each}
    {#if state.dragEdgeStart && state.dragEdgeCurrent}
        {@const p1 = state.getPortPosition(state.dragEdgeStart)}
        {#if p1}
            {@const dBez = getBezierPath(p1, state.dragEdgeCurrent)}
            <path
                d={dBez}
                stroke="#fff"
                stroke-width="2"
                stroke-dasharray="5,5"
                fill="none"
                class="pointer-events-none opacity-50"
            />
        {/if}
    {/if}
</svg>
