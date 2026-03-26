<script lang="ts">
    interface Props {
        id: string;
        path: string;
        color?: string;
        selected?: boolean;
        animated?: boolean;
        onselect?: (e: MouseEvent) => void;
    }

    let { id, path, color = "#555", selected = false, animated = false, onselect }: Props = $props();
</script>

<g class="group/edge">
    <!-- Invisible thicker path for easier clicking/hovering -->
    <path
        d={path}
        stroke="transparent"
        stroke-width="16"
        fill="none"
        class="cursor-pointer pointer-events-auto outline-none transition-colors"
        onclick={onselect}
        role="button"
        tabindex="0"
        onkeydown={() => {}}
    />

    <!-- Visible path -->
    <path
        d={path}
        stroke={selected ? "#fff" : color}
        stroke-width={selected ? "4" : "2"}
        fill="none"
        class="pointer-events-none transition-colors duration-150 shadow-sm {selected ? 'opacity-100' : 'opacity-60 group-hover/edge:opacity-90'}"
        style:stroke-width={selected ? '3px' : '2px'}
    />

    <!-- Animation overlay -->
    {#if animated}
        <path
            d={path}
            stroke="#fff"
            stroke-width="2"
            stroke-dasharray="4 8"
            fill="none"
            class="pointer-events-none animate-flowMove opacity-60"
        />
    {/if}
</g>
