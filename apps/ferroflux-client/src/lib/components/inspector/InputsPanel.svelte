<script lang="ts">
    import { Box } from 'lucide-svelte';
    // In the future, this will subscribe to the 'ActiveWorkflowState' or 'Inbox' content
    const mockInputs = [
        { name: 'webhook.body', type: 'json', value: '{"id": 123}' },
        { name: 'auth.token', type: 'string', value: 'sk_test_...' }
    ];
</script>

<div class="space-y-6">
    <div class="space-y-1">
        <h4 class="text-xs font-bold text-text uppercase tracking-wider">Upstream Variables</h4>
        <p class="text-xs text-text-muted">Data available in the workflow context at this step.</p>
    </div>

    <div class="space-y-2">
        {#each mockInputs as input}
            <div 
                class="bg-bg-input border border-border rounded p-2 flex items-center group cursor-grab active:cursor-grabbing hover:border-text-subtle transition-colors"
                draggable="true"
                ondragstart={(e) => {
                    e.dataTransfer?.setData('text/plain', `{{ ${input.name} }}`);
                    e.dataTransfer?.setData('application/ferroflux-var', input.name);
                }}
            >
                <Box size={14} class="text-brand mr-2" />
                <div class="flex-1 min-w-0">
                    <div class="text-xs font-medium text-text truncate">{input.name}</div>
                    <div class="text-[10px] text-text-subtle font-mono truncate">{input.value}</div>
                </div>
                <div class="text-[10px] bg-bg-sidebar px-1.5 py-0.5 rounded text-text-muted uppercase">{input.type}</div>
            </div>
        {/each}
    </div>
</div>
