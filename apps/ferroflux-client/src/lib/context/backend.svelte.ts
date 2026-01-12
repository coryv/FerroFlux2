import { setContext, getContext } from 'svelte';
import type { IBackend } from '../api/backend';
import { TauriAdapter } from '../api/adapters/tauri';
import { WebAdapter } from '../api/adapters/web';

const BACKEND_KEY = Symbol('BACKEND');

export function initBackendContext() {
    // Detect environment
    // @ts-ignore
    const isTauri = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__;

    const adapter = isTauri ? new TauriAdapter() : new WebAdapter();
    adapter.init();

    setContext(BACKEND_KEY, adapter);
    return adapter;
}

export function getBackend(): IBackend {
    return getContext(BACKEND_KEY);
}
