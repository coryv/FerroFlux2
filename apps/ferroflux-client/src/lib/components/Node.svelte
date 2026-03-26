<script lang="ts">
    import type { NodeData, NodeMetadata } from "$lib/types";
    import { getCategoryColors, getDataTypeColor } from "$lib/utils/nodeColors";
    import { getNodeIcon } from "$lib/utils/nodeIcons";

    interface Props {
        node: NodeData;
        template: NodeMetadata;
        selected?: boolean;
        onPortMouseDown?: (portId: string, event: MouseEvent) => void;
        onPortMouseUp?: (portId: string, event: MouseEvent) => void;
        onmousedown?: (event: MouseEvent) => void;
    }

    let {
        node,
        template,
        selected = false,
        onPortMouseDown,
        onPortMouseUp,
        onmousedown,
    }: Props = $props();

    // Thematic styling based on node category
    let colors = $derived(getCategoryColors(template.category));
    let Icon = $derived(getNodeIcon(template.category, template.name));

    // Collapse state
    let collapsed = $state(false);
</script>

<div
    class="absolute bg-bg-sidebar rounded-xl shadow-xl min-w-[240px] pointer-events-auto select-none transition-[z-index,box-shadow,transform] duration-200"
    class:ring-2={selected}
    class:ring-brand={selected}
    class:ring-offset-2={selected}
    class:ring-offset-bg={selected}
    class:z-50={selected}
    class:z-10={!selected}
    class:shadow-[0_8px_30px_rgb(0,0,0,0.5)]={selected}
    style="left: {node.position.x}px; top: {node.position.y}px;"
    role="button"
    tabindex="0"
    {onmousedown}
>
    <!-- Header -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="{colors.header} p-3 rounded-t-xl border-b {colors.border} flex justify-between items-center group cursor-pointer"
        ondblclick={() => collapsed = !collapsed}
    >
        <div class="flex items-center gap-2">
            <div class="{colors.accent} w-6 h-6 rounded-md flex items-center justify-center text-white shadow-sm">
                <Icon size={14} strokeWidth={2.5} />
            </div>
            <div class="flex flex-col">
                <span class="text-xs font-bold {colors.headerText} tracking-wide">{template.name}</span>
            </div>
        </div>
        <!-- Execution status indicator could go here -->
    </div>

    <!-- Body -->
    {#if !collapsed}
        <div class="p-3 flex flex-col gap-3 animate-fadeIn border border-transparent rounded-b-xl"
             class:border-border={!selected}
             class:border-t-0={true}
        >
            <!-- Inputs -->
            {#if template.inputs.length > 0}
                <div class="flex flex-col gap-1.5">
                    {#each template.inputs as input, i}
                        {@const portId = node.inputs[i]}
                        {@const portColor = getDataTypeColor(input.data_type)}
                        <div class="flex items-center gap-2 h-6 relative group/port">
                            <!-- Input Port -->
                            <div
                                class="absolute -left-[19px] w-3.5 h-3.5 rounded-full border-2 border-bg-sidebar transition-transform hover:scale-125 cursor-crosshair z-10 shadow-sm"
                                style="background-color: {portColor};"
                                title="{input.name} ({input.data_type})"
                                role="button"
                                tabindex="0"
                                onmousedown={(e) => {
                                    e.stopPropagation();
                                    onPortMouseDown?.(portId, e);
                                }}
                                onmouseup={(e) => onPortMouseUp?.(portId, e)}
                            ></div>
                            <span class="text-xs font-medium text-text-muted group-hover/port:text-text transition-colors pointer-events-none">
                                {input.name}
                            </span>
                        </div>
                    {/each}
                </div>
            {/if}

            <!-- Subflow divider -->
            {#if template.inputs.length > 0 && template.outputs.length > 0}
                <div class="h-px w-full bg-border/50"></div>
            {/if}

            <!-- Outputs -->
            {#if template.outputs.length > 0}
                <div class="flex flex-col gap-1.5 items-end">
                    {#each template.outputs as output, i}
                        {@const portId = node.outputs[i]}
                        {@const portColor = getDataTypeColor(output.data_type)}
                        <div class="flex items-center justify-end gap-2 h-6 relative group/port">
                            <span class="text-xs font-medium text-text-muted group-hover/port:text-text transition-colors pointer-events-none">
                                {output.name}
                            </span>
                            <!-- Output Port -->
                            <div
                                class="absolute -right-[19px] w-3.5 h-3.5 rounded-full border-2 border-bg-sidebar transition-transform hover:scale-125 cursor-crosshair z-10 shadow-sm"
                                style="background-color: {portColor};"
                                title="{output.name} ({output.data_type})"
                                role="button"
                                tabindex="0"
                                onmousedown={(e) => {
                                    e.stopPropagation();
                                    onPortMouseDown?.(portId, e);
                                }}
                                onmouseup={(e) => onPortMouseUp?.(portId, e)}
                            ></div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {:else}
        <!-- Collapsed body border -->
        <div class="border-t-0 border border-border rounded-b-xl h-1"></div>
    {/if}
</div>
