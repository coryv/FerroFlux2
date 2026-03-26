<script lang="ts">
    import { Zap } from 'lucide-svelte';
    
    // In the future, this will show the 'Outbox' content or the result of the execution
    const mockOutput = {
        status: 200,
        body: {
            success: true,
            data: ["Item 1", "Item 2"]
        }
    };
</script>

<div class="space-y-6">
    <div class="space-y-1">
        <h4 class="text-xs font-bold text-text uppercase tracking-wider">Node Output</h4>
        <p class="text-xs text-text-muted">The data produced by this node (Outbox).</p>
    </div>

    <div class="bg-bg-input border border-border rounded p-2 font-mono text-xs overflow-x-auto">
        <div class="flex items-center mb-2 pb-2 border-b border-border-subtle">
             <Zap size={12} class="text-status-success mr-2" />
             <span class="text-text-muted">http_response</span>
        </div>
        <pre class="text-text-subtle">{JSON.stringify(mockOutput, null, 2)}</pre>
    </div>
</div>
