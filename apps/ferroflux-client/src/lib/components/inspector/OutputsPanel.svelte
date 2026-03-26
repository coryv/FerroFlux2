<script lang="ts">
    import { Zap, CircleDashed, CheckCircle2 } from 'lucide-svelte';
    import { CanvasState } from '$lib/logic/canvasState.svelte';
    
    let { state: canvasState, node }: { state: CanvasState, node: any } = $props();

    let template = $derived(node && canvasState?.templates[node.data] ? canvasState.templates[node.data] : null);

    let outputs = $derived.by(() => {
        if (!node || !template || !canvasState) return [];
        const templateOutputs = template.outputs || template.interface?.outputs || [];
        
        return (node.outputs || []).map((portId: string, index: number) => {
            const meta = templateOutputs[index] || { name: `output_${index}`, data_type: 'Any' };
            const connections = canvasState.graph.connections.filter((c: any) => c.from === portId);
            
            const downstreamNodes = connections.map((c: any) => {
                const downPort = canvasState.graph.ports[c.to];
                if (downPort) {
                    const downNode = canvasState.graph.nodes[downPort.node_id];
                    if (downNode) {
                        const downTemplate = canvasState.templates[downNode.data];
                        const downNodeConfig = downNode.config || {};
                        const name = downNodeConfig._node_name || downTemplate?.name || downNode.data;
                        
                        const downIndex = downNode.inputs.indexOf(c.to);
                        const downMetaInputs = downTemplate?.inputs || downTemplate?.interface?.inputs || [];
                        const portName = downMetaInputs[downIndex]?.name || `input_${downIndex}`;
                        
                        return { name, portName };
                    }
                }
                return { name: 'Unknown', portName: 'Unknown' };
            });
            
            return {
                id: portId,
                name: meta.name,
                type: meta.data_type,
                connections: downstreamNodes
            };
        });
    });
</script>

<div class="space-y-6 pb-20">
    <div class="space-y-1">
        <h4 class="text-xs font-bold text-text uppercase tracking-wider">Outputs ({outputs.length})</h4>
        <p class="text-[10px] text-text-muted leading-relaxed">Data produced by this node and its connections.</p>
    </div>

    <div class="space-y-2">
        {#each outputs as output}
            <div 
                class="bg-bg-input/20 border border-border/50 rounded-lg p-3 group transition-colors flex flex-col gap-2"
            >
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        {#if output.connections.length > 0}
                            <CheckCircle2 size={14} class="text-status-success" />
                        {:else}
                            <CircleDashed size={14} class="text-text-muted" />
                        {/if}
                        <span class="text-xs font-medium text-text">{output.name}</span>
                    </div>
                    <div class="text-[9px] bg-bg-sidebar/80 px-1.5 py-0.5 rounded text-text-muted uppercase tracking-wider border border-border/30">
                        {output.type}
                    </div>
                </div>
                
                {#if output.connections.length > 0}
                    <div class="flex flex-col gap-1.5 mt-1">
                        {#each output.connections as conn}
                            <div class="flex items-center gap-1.5 text-[10px] text-text-subtle ml-6 bg-bg-sidebar/50 p-1.5 rounded border border-border/20">
                                <Zap size={10} class="text-brand opacity-70" />
                                <span class="truncate">To <span class="text-text-muted font-medium">{conn.name}</span> ({conn.portName})</span>
                            </div>
                        {/each}
                    </div>
                {:else}
                    <div class="text-[10px] text-text-muted/50 ml-6 italic">Not connected</div>
                {/if}
            </div>
        {:else}
             <div class="p-4 border border-border border-dashed rounded-lg flex items-center justify-center text-xs text-text-muted">
                 This node has no outputs.
             </div>
        {/each}
    </div>
</div>
