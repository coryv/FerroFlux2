<script lang="ts">
    interface Props {
        id: string;
        path: string;
        selected?: boolean;
        onselect?: (e: MouseEvent) => void;
    }

    let { id, path, selected = false, onselect }: Props = $props();

    // No derived calc, use path directly
</script>

<path
    d={path}
    stroke="transparent"
    stroke-width="12"
    fill="none"
    class="cursor-pointer pointer-events-auto outline-none"
    onclick={onselect}
    role="button"
    tabindex="0"
    onkeydown={() => {}}
/>

<path
    d={path}
    stroke={selected ? "#3b82f6" : "#555"}
    stroke-width={selected ? "3" : "2"}
    fill="none"
    class="pointer-events-none transition-colors duration-150"
/>
