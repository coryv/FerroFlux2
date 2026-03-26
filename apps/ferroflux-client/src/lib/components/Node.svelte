<script lang="ts">
    import type { NodeData, NodeMetadata } from "$lib/types";

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
</script>

<div
    class="absolute bg-[#1e1e1e] border rounded-md shadow-xl min-w-[200px] pointer-events-auto select-none transition-[z-index]"
    class:border-blue-500={selected}
    class:border-white={!selected}
    class:z-50={selected}
    class:z-10={!selected}
    style="left: {node.position.x}px; top: {node.position.y}px;"
    role="button"
    tabindex="0"
    {onmousedown}
>
    <!-- Header -->
    <div
        class="bg-white/5 p-2 rounded-t-md text-xs font-bold text-neutral-200 border-b border-white/5 flex justify-between items-center"
    >
        <span>{template.name}</span>
        <!-- <span class="text-neutral-500 text-[10px]">{node.id}</span> -->
    </div>

    <!-- Body -->
    <div class="p-2 flex flex-col gap-2">
        <!-- Inputs -->
        {#if template.inputs.length > 0}
            <div class="flex flex-col gap-1">
                {#each template.inputs as input, i}
                    <!-- Input Port ID: node.inputs[i] if valid -->
                    {@const portId = node.inputs[i]}
                    <div class="flex items-center gap-2 h-5 relative">
                        <!-- Input Port -->
                        <div
                            class="absolute -left-[14px] w-3 h-3 rounded-full bg-[#1e1e1e] hover:bg-emerald-400 hover:border-emerald-400 transition-colors cursor-crosshair border z-10"
                            class:border-blue-500={selected}
                            class:border-white={!selected}
                            title={input.data_type}
                            role="button"
                            tabindex="0"
                            onmousedown={(e) => {
                                e.stopPropagation();
                                onPortMouseDown?.(portId, e);
                            }}
                            onmouseup={(e) => onPortMouseUp?.(portId, e)}
                        ></div>
                        <span
                            class="text-xs text-neutral-400 pointer-events-none"
                            >{input.name}</span
                        >
                    </div>
                {/each}
            </div>
        {/if}

        <!-- Outputs -->
        {#if template.outputs.length > 0}
            <div class="flex flex-col gap-1 items-end">
                {#each template.outputs as output, i}
                    {@const portId = node.outputs[i]}
                    <div
                        class="flex items-center justify-end gap-2 h-5 relative"
                    >
                        <span
                            class="text-xs text-neutral-400 pointer-events-none"
                            >{output.name}</span
                        >
                        <!-- Output Port -->
                        <div
                            class="absolute -right-[14px] w-3 h-3 rounded-full bg-[#1e1e1e] hover:bg-emerald-400 hover:border-emerald-400 transition-colors cursor-crosshair border z-10"
                            class:border-blue-500={selected}
                            class:border-white={!selected}
                            title={output.data_type}
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
</div>
