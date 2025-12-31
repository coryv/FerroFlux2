<script lang="ts">
    import {
        calculateBezierPoints,
        calculateLinearPoints,
        getSmartOrthogonalPath,
        type Vec2,
        type Obstacle,
    } from "../utils/routing";

    export let start: Vec2;
    export let end: Vec2;
    export let style: "Linear" | "Cubic" | "Orthogonal" = "Linear";
    export let selected = false;
    export let obstacles: Obstacle[] = [];
    export let buffer = 20;
    export let onclick: (() => void) | null = null;

    let pathPoints: Vec2[] = [];

    $: {
        if (style === "Cubic") {
            const [cp1, cp2] = calculateBezierPoints(start, end);
            pathPoints = [start, cp1, cp2, end];
        } else if (style === "Linear") {
            pathPoints = calculateLinearPoints(start, end);
        } else if (style === "Orthogonal") {
            pathPoints = getSmartOrthogonalPath(start, end, obstacles, buffer);
        }
    }

    function getD(points: Vec2[], style: string) {
        if (points.length < 2) return "";
        if (style === "Cubic") {
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

    $: d = getD(pathPoints, style);
</script>

<g class="edge-group" class:selected>
    <!-- Halo for crossing differentiation -->
    <path
        {d}
        fill="none"
        stroke="var(--canvas-bg, #0f0f11)"
        stroke-width="12"
        stroke-opacity="0.4"
        class="halo"
    />

    <!-- Main visible edge -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <path
        {d}
        fill="none"
        stroke={selected
            ? "var(--edge-selected, #60a5fa)"
            : "var(--edge-color, #4b5563)"}
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="main-path"
        role="button"
        tabindex="0"
        {onclick}
    />
</g>

<style>
    .edge-group {
        pointer-events: none;
    }
    .main-path {
        transition: stroke 0.2s ease;
        pointer-events: visibleStroke;
        cursor: pointer;
    }
    .main-path:hover {
        stroke: var(--edge-hover, #9ca3af) !important;
        stroke-width: 3.5;
    }
    .halo {
        /* This ensures the halo blends with the background */
        mix-blend-mode: normal;
    }
</style>
