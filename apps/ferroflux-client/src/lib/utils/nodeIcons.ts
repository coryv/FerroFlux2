import { Zap, Play, Workflow, Box, Shield, Bot, Code, Database, Globe, SlidersHorizontal, Settings2, Replace } from 'lucide-svelte';

const categoryIcons: Record<string, any> = {
    'Triggers': Zap,
    'Actions': Play,
    'AI & Agents': Bot,
    'Logic': Workflow,
    'Utilities': SlidersHorizontal,
    'Credentials': Shield,
    'default': Box
};

const specificTypeIcons: Record<string, any> = {
    'Script': Code,
    'Transform': Replace,
    'HTTP Request': Globe,
    'Database': Database,
    'Configuration': Settings2
};

export function getNodeIcon(category?: string, name?: string) {
    if (name && specificTypeIcons[name]) {
        return specificTypeIcons[name];
    }
    if (category && categoryIcons[category]) {
        return categoryIcons[category];
    }
    return categoryIcons['default'];
}
