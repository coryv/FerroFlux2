export const categoryColors: Record<string, { header: string, headerText: string, accent: string, border: string }> = {
    'Triggers': {
        header: 'bg-amber-500/10',
        headerText: 'text-amber-400',
        accent: 'bg-amber-500',
        border: 'border-amber-500/20'
    },
    'Actions': {
        header: 'bg-blue-500/10',
        headerText: 'text-blue-400',
        accent: 'bg-blue-500',
        border: 'border-blue-500/20'
    },
    'AI & Agents': {
        header: 'bg-emerald-500/10',
        headerText: 'text-emerald-400',
        accent: 'bg-emerald-500',
        border: 'border-emerald-500/20'
    },
    'Logic': {
        header: 'bg-violet-500/10',
        headerText: 'text-violet-400',
        accent: 'bg-violet-500',
        border: 'border-violet-500/20'
    },
    'Utilities': {
        header: 'bg-slate-500/10',
        headerText: 'text-slate-400',
        accent: 'bg-slate-500',
        border: 'border-slate-500/20'
    },
    'default': {
        header: 'bg-white/5',
        headerText: 'text-neutral-300',
        accent: 'bg-neutral-500',
        border: 'border-white/10'
    }
};

export function getCategoryColors(category?: string) {
    if (!category) return categoryColors['default'];
    return categoryColors[category] || categoryColors['default'];
}

export const dataTypeColors: Record<string, string> = {
    'String': '#14b8a6', // teal-500
    'JSON': '#3b82f6',   // blue-500
    'Boolean': '#eab308', // yellow-500
    'Number': '#a855f7',  // purple-500
    'Any': '#e5e5e5',    // neutral-200
    'default': '#9ca3af' // gray-400
};

export function getDataTypeColor(type?: string) {
    if (!type) return dataTypeColors['default'];
    return dataTypeColors[type] || dataTypeColors['default'];
}
