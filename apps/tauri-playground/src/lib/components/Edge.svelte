<script lang="ts">
    import {
        calculateBezierPoints,
        calculateLinearPoints,
        getSmartOrthogonalPath,
        type Vec2,
        type Obstacle,
    } from "../utils/routing";

    let {
        start,
        end,
        style = "Linear",
        selected = false,
        obstacles = [],
        buffer = 20,
        onclick = null,
    } = $props<{
        start: Vec2;
        end: Vec2;
        style?: "Linear" | "Cubic" | "Orthogonal";
        selected?: boolean;
        obstacles?: Obstacle[];
        buffer?: number;
        onclick?: (() => void) | null;
    }>();

    let pathPoints = $derived.by(() => {
        if (style === "Cubic") {
            const [cp1, cp2] = calculateBezierPoints(start, end);
            return [start, cp1, cp2, end];
        } else if (style === "Linear") {
            return calculateLinearPoints(start, end);
        } else if (style === "Orthogonal") {
            return getSmartOrthogonalPath(start, end, obstacles, buffer);
        }
        return [start, end];
    });

    function getD(points: Vec2[], s: string) {
        if (points.length < 2) return "";
        if (s === "Cubic" && points.length === 4) {
            return `M ${points[0].x} ${points[0].y} C ${points[1].x} ${points[1].y}, ${points[2].x} ${points[2].y}, ${points[3].x} ${points[3].y}`;
        }
        return (
            `M ${points[0].x} ${points[0].y} ` +
            points
                .slice(1)
                .map((p) => `L ${p.x} ${p.y}`)
                .join(" ")
        );
    }

    let d = $derived(getD(pathPoints, style));
</script>

<g class="edge-group" class:selected>
    <!-- Halo for crossing differentiation and easier clicking -->
    <path
        {d}
        fill="none"
        stroke="var(--canvas-bg, #111)"
        stroke-width="12"
        stroke-opacity="0.1"
        class="halo"
    />

    <!-- Selection glow -->
    {#if selected}
        <path
            {d}
            fill="none"
            stroke="#60a5fa"
            stroke-width="6"
            stroke-opacity="0.3"
            class="glow-path"
        />
    {/if}

    <!-- Main visible edge -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <path
        {d}
        fill="none"
        stroke={selected ? "#60a5fa" : "#4b5563"}
        stroke-width={selected ? "3" : "2"}
        stroke-linecap="round"
        stroke-linejoin="round"
        class="main-path"
        class:selected
        role="button"
        tabindex="0"
        onmousedown={(e) => {
            if (onclick) {
                e.stopPropagation();
                onclick();
            }
        }}
    />
</g>

<style>
    .edge-group {
        pointer-events: none;
    }
    .main-path {
        transition:
            stroke 0.2s ease,
            stroke-width 0.2s ease;
        pointer-events: visibleStroke;
        cursor: pointer;
    }
    .main-path:hover {
        stroke: #9ca3af !important;
        stroke-width: 4 !important;
    }
    .main-path.selected:hover {
        stroke: #93c5fd !important;
    }
    .glow-path {
        pointer-events: none;
        filter: blur(2px);
    }
    .halo {
        pointer-events: visibleStroke;
    }
</style>
