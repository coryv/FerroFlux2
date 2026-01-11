<script lang="ts">
    import type { SerializableEdge } from "../../api/adapter";

    let { edge }: { edge: SerializableEdge } = $props();

    let pathData = $derived.by(() => {
        if (!edge || !edge.path || edge.path.length === 0) return "";

        const start = edge.path[0];
        const end = edge.path[edge.path.length - 1];

        if (edge.style === "Cubic" && edge.bezier_control_points) {
            const [cp1, cp2] = edge.bezier_control_points;
            return `M ${start[0]} ${start[1]} C ${cp1[0]} ${cp1[1]}, ${cp2[0]} ${cp2[1]}, ${end[0]} ${end[1]}`;
        } else {
            // Linear or Orthogonal
            return (
                `M ${start[0]} ${start[1]} ` +
                edge.path
                    .slice(1)
                    .map((p) => `L ${p[0]} ${p[1]}`)
                    .join(" ")
            );
        }
    });
</script>

<g class="smart-edge">
    <!-- Clickable transparent path for easier selection -->
    <path d={pathData} stroke="transparent" stroke-width="10" fill="none" />

    <!-- Visible path -->
    <path
        d={pathData}
        stroke="var(--port-color)"
        stroke-width="2"
        fill="none"
        class="edge-line"
    />
</g>

<style>
    .smart-edge {
        cursor: pointer;
        pointer-events: stroke;
    }
    .edge-line {
        transition: stroke 0.2s;
    }
    .smart-edge:hover .edge-line {
        stroke: var(--text-primary);
    }
</style>
