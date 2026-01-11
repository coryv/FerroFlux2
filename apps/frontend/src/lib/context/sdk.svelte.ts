
import { setContext, getContext } from 'svelte';
import { TauriAdapter } from '../api/tauri';
import { WebAdapter } from '../api/web';
import type { BackendAdapter } from '../api/adapter';

const SDK_KEY = Symbol('SDK');

export function initSdkContext() {
    // Basic detection for Tauri environment
    // @ts-ignore
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;

    const adapter = isTauri ? new TauriAdapter() : new WebAdapter();
    setContext(SDK_KEY, adapter);
    return adapter;
}

export function useSdk(): BackendAdapter {
    const adapter = getContext<BackendAdapter>(SDK_KEY);
    if (!adapter) {
        throw new Error('SDK Context not found. call initSdkContext first.');
    }
    return adapter;
}
