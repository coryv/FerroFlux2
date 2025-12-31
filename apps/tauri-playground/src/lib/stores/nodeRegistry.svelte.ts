import type { NodeTemplate } from "$lib/types";

export const nodeRegistry = $state<{
    templates: Record<string, NodeTemplate>;
    loading: boolean;
    error: string | null;
}>({
    templates: {},
    loading: false,
    error: null,
});

export function setTemplates(templates: NodeTemplate[]) {
    const map: Record<string, NodeTemplate> = {};
    for (const t of templates) {
        map[t.id] = t;
    }
    nodeRegistry.templates = map;
    nodeRegistry.loading = false;
}

export function getTemplate(id: string): NodeTemplate | undefined {
    return nodeRegistry.templates[id];
}
