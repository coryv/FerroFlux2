<script lang="ts">
  import { isSidebarOpen } from '$lib/stores';
  import { ChevronsLeft, Settings, Network, Activity, Key, LayoutTemplate, User } from 'lucide-svelte';
  
  let { onOpenWorkflows }: { onOpenWorkflows?: () => void } = $props();

  function toggle() {
    isSidebarOpen.update(v => !v);
  }

  function handleOpenWorkflows() {
    if (onOpenWorkflows) onOpenWorkflows();
    else window.dispatchEvent(new CustomEvent('ferroflux:open-workflow-manager'));
  }
</script>

{#if $isSidebarOpen}
<aside class="w-60 h-full bg-bg-sidebar border-r border-border flex flex-col text-text-muted select-none shadow-xl z-30">
    <!-- Workspace Switcher -->
    <div class="h-14 border-b border-border flex items-center px-4 hover:bg-bg-hover cursor-pointer transition-colors group">
        <div class="w-6 h-6 rounded bg-brand flex items-center justify-center text-xs text-white font-bold shadow-brand/20 shadow-lg">F</div>
        <div class="ml-3 flex flex-col flex-1 truncate">
            <span class="text-sm font-semibold text-text">Default Workspace</span>
            <span class="text-[10px] text-text-subtle">Personal</span>
        </div>
        <button class="opacity-0 group-hover:opacity-100 p-1 hover:bg-bg-active rounded text-text-subtle hover:text-text transition-all" onclick={toggle} title="Close sidebar">
            <ChevronsLeft size={16} />
        </button>
    </div>

    <!-- Primary Navigation -->
    <div class="flex flex-col p-3 space-y-1">
        <button 
            onclick={handleOpenWorkflows} 
            class="flex items-center px-3 py-2 bg-bg-active hover:bg-bg-hover rounded-md text-sm group text-left text-text font-medium shadow-sm border border-border transition-colors"
        >
            <Network size={16} class="mr-3 text-brand" />
            <span class="flex-1">Workflows</span>
        </button>
        <button class="flex items-center px-3 py-2 hover:bg-bg-hover rounded-md text-sm text-left text-text-subtle hover:text-text transition-colors">
            <Activity size={16} class="mr-3" />
            <span>Executions</span>
        </button>
        <button class="flex items-center px-3 py-2 hover:bg-bg-hover rounded-md text-sm text-left text-text-subtle hover:text-text transition-colors">
            <Key size={16} class="mr-3" />
            <span>Credentials</span>
        </button>
        <button class="flex items-center px-3 py-2 hover:bg-bg-hover rounded-md text-sm text-left text-text-subtle hover:text-text transition-colors disabled opacity-50 cursor-not-allowed">
            <LayoutTemplate size={16} class="mr-3" />
            <span>Templates</span>
        </button>
    </div>

    <div class="flex-1"></div>
    
    <!-- Footer -->
    <div class="p-3 border-t border-border flex flex-col gap-1">
        <button class="flex items-center px-3 py-2 hover:bg-bg-hover rounded-md text-sm text-left text-text-subtle hover:text-text transition-colors">
            <Settings size={16} class="mr-3" />
            <span>Settings</span>
        </button>
        <button class="flex items-center px-3 py-2 hover:bg-bg-hover rounded-md text-sm text-left text-text-subtle hover:text-text transition-colors">
            <User size={16} class="mr-3" />
            <span>My Account</span>
        </button>
    </div>
</aside>
{/if}
