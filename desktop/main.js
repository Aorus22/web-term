const { app, BrowserWindow, ipcMain, Menu } = require('electron');
const path = require('path');
const { spawn } = require('child_process');
const fs = require('fs');

let mainWindow;
let backendProcess;

// Remove the default menu (File, Edit, etc.)
Menu.setApplicationMenu(null);

function startBackend() {
    const isDev = process.env.NODE_ENV === 'development';
    
    let backendPath;
    if (app.isPackaged) {
        backendPath = path.join(process.resourcesPath, 'be', 'webterm-backend');
        if (process.platform === 'win32') {
            backendPath += '.exe';
        }
    } else {
        backendPath = path.join(__dirname, '..', 'be', 'webterm-backend');
        if (process.platform === 'win32') {
            backendPath += '.exe';
        }
    }

    // Ensure the data directory exists in userData
    const userDataPath = app.getPath('userData');
    const dbPath = path.join(userDataPath, 'webterm.db');
    
    console.log(`Starting backend: ${backendPath}`);
    console.log(`DB Path: ${dbPath}`);

    const env = {
        ...process.env,
        WEBTERM_PORT: ':8080',
        WEBTERM_DB_PATH: dbPath,
        WEBTERM_ALLOWED_ORIGINS: '*', // For electron, allow all or specific file://
    };

    backendProcess = spawn(backendPath, [], { env });

    backendProcess.stdout.on('data', (data) => {
        console.log(`Backend: ${data}`);
    });

    backendProcess.stderr.on('data', (data) => {
        console.error(`Backend Error: ${data}`);
    });

    backendProcess.on('close', (code) => {
        console.log(`Backend process exited with code ${code}`);
    });
}

function createWindow() {
    const isWin = process.platform === 'win32';
    const isMac = process.platform === 'darwin';

    mainWindow = new BrowserWindow({
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        frame: false, // Frameless for all platforms
        transparent: false,
        // titleBarStyle is ONLY for macOS and Windows. 
        // Applying it on Linux can cause some window managers to force decorations.
        ...( (isWin || isMac) ? { titleBarStyle: 'hidden' } : {} ),
        titleBarOverlay: isWin ? {
            color: '#00000000', 
            symbolColor: '#94a3b8',
            height: 48 
        } : false,
        autoHideMenuBar: true, // Specifically for Linux to prevent menu-triggered decorations
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        },
        title: "WebTerm Desktop"
    });

    const isDev = process.env.NODE_ENV === 'development';
    
    if (isDev) {
        mainWindow.loadURL('http://localhost:5173');
    } else if (app.isPackaged) {
        mainWindow.loadFile(path.join(process.resourcesPath, 'fe', 'dist', 'index.html'));
    } else {
        mainWindow.loadFile(path.join(__dirname, '..', 'fe', 'dist', 'index.html'));
    }

    mainWindow.on('closed', function () {
        mainWindow = null;
    });

    mainWindow.on('maximize', () => {
        mainWindow.webContents.send('window-state-change', 'maximized');
    });

    mainWindow.on('unmaximize', () => {
        mainWindow.webContents.send('window-state-change', 'restored');
    });
}

// IPC Handlers for Window Controls
ipcMain.on('window-minimize', () => {
    mainWindow.minimize();
});

ipcMain.on('window-maximize', () => {
    if (mainWindow.isMaximized()) {
        mainWindow.unmaximize();
    } else {
        mainWindow.maximize();
    }
});

ipcMain.on('window-close', () => {
    mainWindow.close();
});

ipcMain.handle('get-window-state', () => {
    return mainWindow ? (mainWindow.isMaximized() ? 'maximized' : 'restored') : 'restored';
});

app.on('ready', () => {
    startBackend();
    createWindow();
});

app.on('window-all-closed', function () {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});

app.on('activate', function () {
    if (mainWindow === null) {
        createWindow();
    }
});

app.on('will-quit', () => {
    if (backendProcess) {
        backendProcess.kill();
    }
});
