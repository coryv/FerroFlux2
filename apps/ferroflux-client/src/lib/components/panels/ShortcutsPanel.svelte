<script lang="ts">
    import { X } from "lucide-svelte";
    import { fly } from "svelte/transition";

    let isOpen = $state(false);

    export function toggle() {
        isOpen = !isOpen;
    }

    function handleKeydown(e: KeyboardEvent) {
        // Shift + ? is usually just '?' with shift modifier, but e.key is '?'
        if (e.key === "?" && document.activeElement?.tagName !== "INPUT" && document.activeElement?.tagName !== "TEXTAREA") {
            e.preventDefault();
            toggle();
        } else if (e.key === "Escape" && isOpen) {
            isOpen = false;
        }
    }

    const shortcutGroups = [
        {
            title: "Navigation & View",
            items: [
                { keys: ["⌘", "K"], label: "Open Command Palette" },
                { keys: ["?"], label: "Toggle Shortcuts Panel" },
                { keys: ["⌘", "+"], label: "Zoom In" },
                { keys: ["⌘", "-"], label: "Zoom Out" },
                { keys: ["⌘", "1"], label: "Fit to View" },
            ]
        },
        {
            title: "Canvas Editing",
            items: [
                { keys: ["⌘", "A"], label: "Select All" },
                { keys: ["⌘", "C"], label: "Copy Selected" },
                { keys: ["⌘", "V"], label: "Paste" },
                { keys: ["⌘", "D"], label: "Duplicate Selected" },
                { keys: ["Backspace", "or", "Delete"], label: "Delete Selected" },
            ]
        },
        {
            title: "History",
            items: [
                { keys: ["⌘", "Z"], label: "Undo" },
                { keys: ["⌘", "⇧", "Z"], label: "Redo" },
            ]
        },
        {
            title: "Workflow",
            items: [
                { keys: ["⌘", "S"], label: "Save Workflow" },
                { keys: ["⌘", "Enter"], label: "Execute Workflow" },
            ]
        }
    ];
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="fixed inset-0 z-[100] bg-black/50 backdrop-blur-sm flex items-center justify-center p-6" onclick={() => isOpen = false}>
        <div 
            class="bg-bg-sidebar border border-white/10 shadow-2xl rounded-xl w-full max-w-3xl overflow-hidden text-text flex flex-col"
            onclick={(e) => e.stopPropagation()}
            transition:fly={{ y: 20, duration: 250 }}
        >
            <div class="flex items-center justify-between p-6 border-b border-white/5 bg-white/5">
                <div>
                    <h2 class="text-xl font-bold">Keyboard Shortcuts</h2>
                    <p class="text-sm text-text-muted mt-1">Master FerroFlux with these quick commands.</p>
                </div>
                <button onclick={() => isOpen = false} class="p-2 hover:bg-white/10 rounded-full transition-colors text-text-subtle hover:text-text">
                    <X size={20} />
                </button>
            </div>

            <div class="p-6 grid grid-cols-1 md:grid-cols-2 gap-8">
                {#each shortcutGroups as group}
                    <div>
                        <h3 class="text-sm font-bold text-text-muted uppercase tracking-wider mb-4">{group.title}</h3>
                        <div class="space-y-3">
                            {#each group.items as item}
                                <div class="flex items-center justify-between">
                                    <span class="text-sm font-medium">{item.label}</span>
                                    <div class="flex items-center gap-1">
                                        {#each item.keys as key}
                                            <kbd class="px-2 py-1 bg-bg-input border border-border rounded text-xs font-mono font-bold text-text-muted shadow-sm">{key}</kbd>
                                        {/each}
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    </div>
{/if}
