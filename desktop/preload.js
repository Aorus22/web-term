const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
    minimize: () => ipcRenderer.send('window-minimize'),
    maximize: () => ipcRenderer.send('window-maximize'),
    close: () => ipcRenderer.send('window-close'),
    isElectron: true
});

window.addEventListener('DOMContentLoaded', () => {
    console.log('WebTerm Desktop Preload Loaded');
});
