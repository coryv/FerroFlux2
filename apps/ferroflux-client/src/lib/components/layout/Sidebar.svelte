<script lang="ts">
  import { isSidebarOpen } from '$lib/stores';
  import { ChevronsLeft, Plus, Search, Settings, Home, PlayCircle, Key, LayoutTemplate, Network, Activity } from 'lucide-svelte';
  
  // Mock data for automation tool context
  const recentWorkflows = ['Data Pipeline Alpha', 'Email Triager Agent', 'Crypto Sentiment Bot'];
  const folders = ['Marketing Automation', 'DevOps Tools', 'Personal Assistants'];
  
  function toggle() {
    isSidebarOpen.update(v => !v);
  }
</script>

{#if $isSidebarOpen}
<aside class="w-60 h-full bg-bg-sidebar border-r border-border flex flex-col text-text-muted select-none">
    <!-- Workspace Switcher -->
    <div class="h-12 flex items-center px-3 hover:bg-bg-hover cursor-pointer transition-colors group">
        <div class="w-5 h-5 rounded bg-brand mr-2 flex items-center justify-center text-[10px] text-white font-bold">F</div>
        <span class="text-sm font-medium text-text truncate flex-1">FerroFlux Workspace</span>
        <button class="opacity-0 group-hover:opacity-100 p-1 hover:bg-bg-active rounded text-text-subtle hover:text-text" onclick={toggle} title="Close sidebar">
            <ChevronsLeft size={16} />
        </button>
    </div>

    <!-- Primary Navigation -->
    <div class="flex flex-col px-2 py-2 space-y-0.5">
        <button class="flex items-center px-2 py-1 hover:bg-bg-hover rounded text-sm group text-left text-text">
            <Search size={16} class="mr-2" />
            <span class="flex-1">Search</span>
            <span class="text-xs text-text-subtle border border-border-subtle px-1 rounded bg-bg-input">⌘K</span>
        </button>
        
        <div class="pt-2 pb-1 px-2 text-[11px] font-bold text-text-subtle uppercase tracking-wider">Platform</div>
        
        <button class="flex items-center px-2 py-1 hover:bg-bg-hover rounded text-sm text-left text-text-muted hover:text-text">
            <Network size={16} class="mr-2" />
            <span>Workflows</span>
        </button>
        <button class="flex items-center px-2 py-1 hover:bg-bg-hover rounded text-sm text-left text-text-muted hover:text-text">
            <Activity size={16} class="mr-2" />
            <span>Executions</span>
        </button>
        <button class="flex items-center px-2 py-1 hover:bg-bg-hover rounded text-sm text-left text-text-muted hover:text-text">
            <Key size={16} class="mr-2" />
            <span>Credentials</span>
        </button>
        <button class="flex items-center px-2 py-1 hover:bg-bg-hover rounded text-sm text-left text-text-muted hover:text-text">
            <LayoutTemplate size={16} class="mr-2" />
            <span>Templates</span>
        </button>
         <button class="flex items-center px-2 py-1 hover:bg-bg-hover rounded text-sm text-left text-text-muted hover:text-text">
            <Settings size={16} class="mr-2" />
            <span>Settings</span>
        </button>
    </div>

    <!-- Scroller -->
    <div class="flex-1 overflow-y-auto px-2 py-2 scrollbar-hide">
        <!-- Recent Workflows -->
        <div class="mb-4">
            <div class="px-2 text-xs font-semibold text-text-subtle mb-1">Recent</div>
            {#each recentWorkflows as item}
                <button class="flex items-center w-full px-2 py-1 hover:bg-bg-hover rounded text-sm text-text-muted hover:text-text text-left">
                    <span class="mr-2 text-xs opacity-70">⚡</span>
                    {item}
                </button>
            {/each}
        </div>

        <!-- Folders / Projects -->
         <div>
            <div class="px-2 text-xs font-semibold text-text-subtle mb-1 flex items-center group">
                <span>Projects</span>
                <button class="ml-auto opacity-0 group-hover:opacity-100 hover:bg-bg-active p-0.5 rounded text-text hover:text-text">
                     <Plus size={14} />
                </button>
            </div>
            {#each folders as item}
                <button class="flex items-center w-full px-2 py-1 hover:bg-bg-hover rounded text-sm text-text-muted hover:text-text text-left">
                    <span class="mr-2 text-xs opacity-70">📁</span>
                    {item}
                </button>
            {/each}
        </div>
    </div>
    
    <!-- Footer -->
    <div class="p-2 border-t border-border-subtle">
        <button class="flex items-center w-full px-2 py-1 hover:bg-bg-hover rounded text-sm text-left text-brand hover:text-brand-hover transition-colors">
            <Plus size={16} class="mr-2" />
            <span class="font-medium">New Workflow</span>
        </button>
    </div>
</aside>
{/if}
