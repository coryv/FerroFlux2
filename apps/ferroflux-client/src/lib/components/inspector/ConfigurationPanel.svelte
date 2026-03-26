<script lang="ts">
    import FieldRenderer from './FieldRenderer.svelte';
    import { CanvasState } from '$lib/logic/canvasState.svelte';

    let { template, node, state: canvasState, config = $bindable() }: { template: any, node: any, state: CanvasState, config?: Record<string, any> } = $props();

    // The parent provides the config object
    if (!config) config = {};
    let loading = $state(true);

    // Sync from backend when node changes
    $effect(() => {
        if (node && canvasState) {
            loading = true;
            // The template provides the schema (settings)
            
            // Fetch real config
            canvasState.backend.getNodeConfig(node.id).then((serverConfig) => {
                // Initialize default values for missing keys
                const newConfig = { ...serverConfig };
                const settingsDef = template.settings || template.interface?.settings || [];
                
                settingsDef.forEach((setting: any) => {
                    if (newConfig[setting.name] === undefined && setting.default !== undefined) {
                        newConfig[setting.name] = setting.default;
                    }
                });
                
                // Copy properties over to preserve reference for bindable
                for (const k in newConfig) {
                    config![k] = newConfig[k];
                }
                
                loading = false;
            }).catch(e => {
                console.error("Failed to fetch node config", e);
                loading = false;
            });
        }
    });

    // Handle field updates
    function handleFieldChange(key: string, value: any) {
        if(config) {
            config[key] = value;
        }
        // Debounce backend update
        saveToBackend(key, value);
    }
    
    let saveTimeout: ReturnType<typeof setTimeout>;
    function saveToBackend(key: string, value: any) {
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
            canvasState.backend.updateNodeConfig(node.id, key, value)
                .catch(e => console.error("Failed to save config value", e));
        }, 300);
    }

    let settings = $derived(template.settings || template.interface?.settings || []);

    function evaluateCondition(condition: string, context: any): boolean {
        if (!condition) return true;
        try {
            const keys = Object.keys(context);
            const values = Object.values(context);
            const fn = new Function(...keys, `return ${condition};`);
            return !!fn(...values);
        } catch (e) {
            console.warn(`Failed to evaluate condition: "${condition}"`, e);
            return true;
        }
    }
</script>

{#if loading}
    <div class="flex items-center justify-center p-8 text-text-muted">
        <div class="w-4 h-4 border-2 border-brand border-t-transparent rounded-full animate-spin mr-2"></div>
        <span class="text-xs">Loading configuration...</span>
    </div>
{:else}
    <div class="space-y-6 pb-20">
        <div class="space-y-1">
            <h4 class="text-xs font-bold text-text uppercase tracking-wider">Node Configuration</h4>
            <p class="text-[10px] text-text-muted leading-relaxed">Configure the static parameters for this action.</p>
        </div>

        {#if settings.length > 0}
            <div class="space-y-4">
                {#each settings as setting}
                    {#if !setting.show_if || evaluateCondition(setting.show_if, config)}
                        <div class="space-y-1.5 p-3 rounded-lg border border-border/50 bg-bg-input/20">
                            <label class="block space-y-2">
                                <span class="text-xs font-medium text-text-subtle">
                                    {setting.label || setting.name}
                                    {#if setting.required}<span class="text-brand ml-1">*</span>{/if}
                                </span>
                                <FieldRenderer 
                                    definition={setting} 
                                    value={config ? config[setting.name] : undefined} 
                                    onValueChange={(v: any) => handleFieldChange(setting.name, v)}
                                />
                            </label>
                            {#if setting.description}
                                <p class="text-[10px] text-text-muted mt-1 leading-relaxed">{setting.description}</p>
                            {/if}
                        </div>
                    {/if}
                {/each}
            </div>
        {:else}
             <div class="p-4 border border-border border-dashed rounded-lg flex items-center justify-center text-xs text-text-muted">
                 No configuration available for this node type.
             </div>
        {/if}
    </div>
{/if}
