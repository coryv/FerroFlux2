<script lang="ts">
    import { isSidebarOpen } from '$lib/stores';
    import { Menu, MoreHorizontal, Star, Share } from 'lucide-svelte';
    
    function toggle() {
        isSidebarOpen.update(v => !v);
    }
</script>

<header class="h-12 border-b border-border flex items-center px-4 bg-bg select-none" data-tauri-drag-region>
    <!-- Left: Navigation / Sidebar Toggle -->
    <div class="flex items-center">
        {#if !$isSidebarOpen}
            <button onclick={toggle} class="p-1 hover:bg-bg-hover rounded mr-2 text-text-subtle hover:text-text transition-colors">
                <Menu size={18} />
            </button>
        {/if}
        
        <!-- Breadcrumbs -->
        <div class="flex items-center text-sm text-text-muted">
            <span class="hover:underline cursor-pointer hover:text-text transition-colors">FerroFlux Architecture</span>
            <span class="mx-2 text-text-subtle">/</span>
            <span class="text-text font-medium cursor-pointer">Canvas View</span>
        </div>
    </div>

    <!-- Right: Actions -->
    <div class="ml-auto flex items-center space-x-1">
        <button class="p-1 hover:bg-bg-hover rounded text-text-subtle hover:text-text transition-colors">
            <Share size={16} />
        </button>
        <button class="p-1 hover:bg-bg-hover rounded text-text-subtle hover:text-text transition-colors">
            <Star size={16} />
        </button>
        <button class="p-1 hover:bg-bg-hover rounded text-text-subtle hover:text-text transition-colors">
            <MoreHorizontal size={16} />
        </button>
    </div>
</header>
