export const workflowContext = $state<{
    id: string;
    name: string;
}>({
    id: crypto.randomUUID(),
    name: "New Workflow",
});

export function resetWorkflowId() {
    workflowContext.id = crypto.randomUUID();
}
