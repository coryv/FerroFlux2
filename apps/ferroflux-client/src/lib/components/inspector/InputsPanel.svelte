<script lang="ts">
    import { Box, CheckCircle2, CircleDashed } from 'lucide-svelte';
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    
    let { state: canvasState, node }: { state: CanvasState, node: any } = $props();

    let template = $derived(node && canvasState?.templates[node.data] ? canvasState.templates[node.data] : null);

    let inputs = $derived.by(() => {
        if (!node || !template || !canvasState) return [];
        const templateInputs = template.inputs || template.interface?.inputs || [];
        
        return (node.inputs || []).map((portId: string, index: number) => {
            const meta = templateInputs[index] || { name: `input_${index}`, data_type: 'Any' };
            const connection = canvasState.graph.connections.find((c: any) => c.to === portId);
            let upstreamNodeName = null;
            let upstreamPortName = null;
            
            if (connection) {
                const upPort = canvasState.graph.ports[connection.from];
                if (upPort) {
                    const upNode = canvasState.graph.nodes[upPort.node_id];
                    if (upNode) {
                        const upTemplate = canvasState.templates[upNode.data];
                        const upNodeConfig = upNode.config || {};
                        upstreamNodeName = upNodeConfig._node_name || upTemplate?.name || upNode.data;
                        
                        const upIndex = upNode.outputs.indexOf(connection.from);
                        const upMetaOutputs = upTemplate?.outputs || upTemplate?.interface?.outputs || [];
                        upstreamPortName = upMetaOutputs[upIndex]?.name || `output_${upIndex}`;
                    }
                }
            }
            
            return {
                id: portId,
                name: meta.name,
                type: meta.data_type,
                connected: !!connection,
                upstreamNodeName,
                upstreamPortName
            };
        });
    });
</script>

<div class="space-y-6 pb-20">
    <div class="space-y-1">
        <h4 class="text-xs font-bold text-text uppercase tracking-wider">Inputs ({inputs.length})</h4>
        <p class="text-[10px] text-text-muted leading-relaxed">Data required by this node from upstream connections.</p>
    </div>

    <div class="space-y-2">
        {#each inputs as input}
            <div 
                class="bg-bg-input/20 border border-border/50 rounded-lg p-3 group transition-colors flex flex-col gap-2"
            >
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        {#if input.connected}
                            <CheckCircle2 size={14} class="text-status-success" />
                        {:else}
                            <CircleDashed size={14} class="text-text-muted" />
                        {/if}
                        <span class="text-xs font-medium text-text">{input.name}</span>
                    </div>
                    <div class="text-[9px] bg-bg-sidebar/80 px-1.5 py-0.5 rounded text-text-muted uppercase tracking-wider border border-border/30">
                        {input.type}
                    </div>
                </div>
                
                {#if input.connected}
                    <div class="flex items-center gap-1.5 text-[10px] text-text-subtle ml-6 bg-bg-sidebar/50 p-1.5 rounded border border-border/20">
                        <Box size={10} class="text-brand opacity-70" />
                        <span class="truncate">From <span class="text-text-muted font-medium">{input.upstreamNodeName}</span> ({input.upstreamPortName})</span>
                    </div>
                {:else}
                    <div class="text-[10px] text-text-muted/50 ml-6 italic">Not connected</div>
                {/if}
            </div>
        {:else}
             <div class="p-4 border border-border border-dashed rounded-lg flex items-center justify-center text-xs text-text-muted">
                 This node has no inputs.
             </div>
        {/each}
    </div>
</div>
