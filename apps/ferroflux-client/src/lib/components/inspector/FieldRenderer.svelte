<script lang="ts">
    let { definition, value = $bindable(), onValueChange }: { definition: any, value?: any, onValueChange?: (v: any) => void } = $props();

    function triggerChange() {
        if (onValueChange) {
            // Give Svelte binding a microtick to update value before calling
            setTimeout(() => onValueChange(value), 0);
        }
    }

    function onDrop(e: DragEvent) {
        const text = e.dataTransfer?.getData('text/plain');
        if (text) {
            e.preventDefault();
            // Simple append for now, could be improved to insert at cursor
            // Check if element is focused to insert at cursor?
            const target = e.target as HTMLInputElement | HTMLTextAreaElement;
            if (target) {
                const start = target.selectionStart || 0;
                const end = target.selectionEnd || 0;
                const current = value ? String(value) : '';
                value = current.slice(0, start) + text + current.slice(end);
                
                // Restore focus/cursor (requires tick or slight delay in Svelte)
                setTimeout(() => {
                    target.focus();
                    target.setSelectionRange(start + text.length, start + text.length);
                }, 0);
            } else {
                value = (value || '') + text;
            }
        }
    }
</script>

{#if definition.type === 'string' || definition.type === 'number' || definition.type === 'time'}
    <input 
        type={definition.type === 'string' ? 'text' : definition.type} 
        bind:value 
        class="w-full bg-bg-input border border-border rounded px-2 py-1.5 text-xs text-text focus:outline-none focus:border-brand transition-colors placeholder:text-text-muted"
        placeholder={definition.placeholder || ''}
        min={definition.min}
        max={definition.max}
        ondrop={onDrop}
        ondragover={(e) => e.preventDefault()}
        oninput={triggerChange}
    />

{:else if definition.type === 'select' || definition.type === 'connection_select'}
    <div class="relative">
        <select 
            bind:value 
            onchange={triggerChange}
            class="w-full bg-bg-input border border-border rounded px-2 py-1.5 text-xs text-text focus:outline-none focus:border-brand transition-colors appearance-none"
        >
            {#if definition.options}
                {#each definition.options as opt}
                    {#if typeof opt === 'object' && opt !== null && 'value' in opt}
                        <option value={opt.value}>{opt.label || opt.value}</option>
                    {:else}
                        <option value={opt}>{opt}</option>
                    {/if}
                {/each}
            {:else}
                <option disabled>No options available</option>
            {/if}
        </select>
        <!-- Custom Arrow -->
        <div class="absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none text-text-subtle">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
        </div>
    </div>

{:else if definition.type === 'textarea'}
    <textarea 
        bind:value
        rows="4"
        class="w-full bg-bg-input border border-border rounded px-2 py-1.5 text-xs text-text focus:outline-none focus:border-brand transition-colors font-mono resize-y"
        ondrop={onDrop}
        ondragover={(e) => e.preventDefault()}
        oninput={triggerChange}
    ></textarea>

{:else if definition.type === 'multi-select'}
    <!-- Simple placeholder for multi-select -->
    <div class="text-xs text-text-muted italic border border-border rounded px-2 py-1.5 bg-bg-input">
        Multi-select not yet implemented
    </div>

{:else if definition.type === 'boolean'}
    <label class="relative inline-flex items-center cursor-pointer">
        <input type="checkbox" bind:checked={value} class="sr-only peer" onchange={triggerChange}>
        <div class="w-9 h-5 bg-bg-sidebar peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-text after:border-border after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-brand"></div>
    </label>

{:else if definition.type === 'json' || definition.type === 'code' || definition.type === 'expression'}
    <div class="relative group">
        <textarea 
            bind:value
            rows={definition.type === 'expression' ? 2 : 6}
            class="w-full bg-[#1e1e1e] border border-[#333] rounded px-3 py-2 text-xs focus:outline-none focus:border-brand transition-colors font-mono resize-y leading-relaxed"
            placeholder={definition.type === 'json' ? '{\n  "key": "value"\n}' : definition.type === 'expression' ? '{{ $json.body }}' : '// Write your script here'}
            ondrop={onDrop}
            ondragover={(e) => e.preventDefault()}
            oninput={triggerChange}
            style="color: #d4d4d4;"
        ></textarea>
        <div class="absolute right-2 bottom-2 text-[9px] text-[#888] pointer-events-none uppercase bg-black/80 px-1.5 py-0.5 rounded shadow-sm opacity-50 group-hover:opacity-100 transition-opacity">
            {definition.type}
        </div>
    </div>

{:else}
    <div class="text-xs text-status-warning">Unknown field type: {definition.type}</div>
{/if}
