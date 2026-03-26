<script lang="ts">
    import FieldRenderer from './FieldRenderer.svelte';

    let { template, node } = $props();

    // Use state for the local form values
    let config = $state({});

    // Sync local state when node changes
    $effect(() => {
        config = { ...node.data };
    });

    // Helper to normalize settings path
    let settings = $derived(template.settings || template.interface?.settings || []);

    // Simple expression evaluator for show_if
    function evaluateCondition(condition: string, context: any): boolean {
        if (!condition) return true;
        try {
            // Create a safe-ish evaluation context
            // In production, use a proper expression parser like 'jsep' or 'filtrex'
            // For this prototype, we'll use a Function with strict variable access
            const keys = Object.keys(context);
            const values = Object.values(context);
            const fn = new Function(...keys, `return ${condition};`);
            return !!fn(...values);
        } catch (e) {
            console.warn(`Failed to evaluate condition: "${condition}"`, e);
            return true; // Show on error fallback
        }
    }

    // Effect to update node data when config changes (Debounced or on save in a real app)
    $effect(() => {
        // node.data = { ...config }; 
    });
</script>

<div class="space-y-6">
    <div class="space-y-1">
        <h4 class="text-xs font-bold text-text uppercase tracking-wider">Node Settings</h4>
        <p class="text-xs text-text-muted">Configure the static parameters for this action.</p>
    </div>

    <div class="space-y-4">
        {#each settings as setting}
            {#if !setting.show_if || evaluateCondition(setting.show_if, config)}
                <div class="space-y-1.5">
                    <label class="block space-y-1.5">
                        <span class="text-xs font-medium text-text-subtle">
                            {setting.label}
                            {#if setting.required}<span class="text-brand">*</span>{/if}
                        </span>
                        <FieldRenderer 
                            definition={setting} 
                            bind:value={config[setting.name]} 
                        />
                    </label>
                    {#if setting.description}
                        <p class="text-[10px] text-text-muted">{setting.description}</p>
                    {/if}
                </div>
            {/if}
        {/each}
    </div>
</div>
