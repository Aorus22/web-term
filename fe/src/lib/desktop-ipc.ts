import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export const isElectron = !!window.electron;
export const isTauri = !!(window as any).__TAURI_INTERNALS__;
export const isDesktop = isElectron || isTauri;

export async function startDragging() {
  if (isTauri) {
    const window = getCurrentWebviewWindow();
    await window.startDragging();
  }
}

export async function startResizing(direction: string) {
  if (isTauri) {
    const window = getCurrentWebviewWindow();
    // @ts-ignore - Tauri 2.0 uses string directions
    await window.startResizing(direction);
  }
}

export async function minimizeWindow() {
  if (isElectron) {
    window.electron?.minimize();
  } else if (isTauri) {
    await invoke('minimize_window');
  }
}

export async function maximizeWindow() {
  if (isElectron) {
    window.electron?.maximize();
  } else if (isTauri) {
    await invoke('maximize_window');
  }
}

export async function closeWindow() {
  if (isElectron) {
    window.electron?.close();
  } else if (isTauri) {
    await invoke('close_window');
  }
}

export async function getWindowState(): Promise<'maximized' | 'restored'> {
  if (isElectron) {
    return (await window.electron?.getWindowState()) || 'restored';
  } else if (isTauri) {
    return await invoke<'maximized' | 'restored'>('get_window_state');
  }
  return 'restored';
}

export function onWindowStateChange(callback: (state: 'maximized' | 'restored') => void) {
  if (isElectron) {
    window.electron?.onWindowStateChange(callback);
    return () => {};
  } else if (isTauri) {
    const unlisten = listen<'maximized' | 'restored'>('window-state-change', (event) => {
      callback(event.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }
  return () => {};
}

export async function getBackendPort(): Promise<number> {
  if (isElectron) {
    return (await (window as any).electron?.getBackendPort()) || 8080;
  } else if (isTauri) {
    return await invoke<number>('get_backend_port');
  }
  return 8080;
}

export function onBackendReady(callback: (port: number) => void) {
  if (isElectron) {
    (window as any).electron?.onBackendReady(callback);
    return () => {};
  } else if (isTauri) {
    const unlisten = listen<number>('backend-ready', (event) => {
      callback(event.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }
  return () => {};
}

export const platform = isElectron ? window.electron?.platform : (isTauri ? 'linux' : 'web');
