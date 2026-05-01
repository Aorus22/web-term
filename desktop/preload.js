const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
    minimize: () => ipcRenderer.send('window-minimize'),
    maximize: () => ipcRenderer.send('window-maximize'),
    close: () => ipcRenderer.send('window-close'),
    getWindowState: () => ipcRenderer.invoke('get-window-state'),
    onWindowStateChange: (callback) => ipcRenderer.on('window-state-change', (event, state) => callback(state)),
    isElectron: true,
    platform: process.platform
});

window.addEventListener('DOMContentLoaded', () => {
    console.log('WebTerm Desktop Preload Loaded');
});
