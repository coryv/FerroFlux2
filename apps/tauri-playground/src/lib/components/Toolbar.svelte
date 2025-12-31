<script lang="ts">
    let { status, onRefresh, onAddNode, onDeploy } = $props<{
        status: string;
        onRefresh: () => void;
        onAddNode: () => void;
        onDeploy: () => void;
    }>();
</script>

<div class="toolbar">
    <span class="status">Status: {status}</span>
    <button onclick={onRefresh}>Refresh</button>
    <button onclick={onAddNode}>Add Node</button>
    <button onclick={onDeploy} class="deploy-btn">Deploy to Engine</button>
</div>

<style>
    .toolbar {
        padding: 10px;
        background: #1e1e1e;
        color: #ccc;
        display: flex;
        gap: 10px;
        align-items: center;
        border-bottom: 1px solid #333;
        height: 40px;
        z-index: 100;
        position: relative;
    }
    .status {
        margin-right: auto;
        font-family: monospace;
        color: #888;
    }
    button {
        background: #333;
        color: white;
        border: 1px solid #444;
        padding: 5px 10px;
        border-radius: 4px;
        cursor: pointer;
    }
    button:hover {
        background: #444;
    }
    .deploy-btn {
        background: #2a6;
        border-color: #285;
        color: white;
    }
    .deploy-btn:hover {
        background: #3b7;
    }
</style>
