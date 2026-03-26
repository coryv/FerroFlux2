<script lang="ts">
    import Sidebar from './Sidebar.svelte';
    import TopBar from './TopBar.svelte';
    
    let { children } = $props();
</script>

<div class="flex h-screen w-screen bg-bg text-text overflow-hidden font-sans">
    <Sidebar />
    
    <div class="flex-1 flex flex-col h-full overflow-hidden">
        <TopBar />
        <main class="flex-1 overflow-hidden relative bg-bg">
             {@render children()}
        </main>
    </div>
</div>
