const { app, BrowserWindow, ipcMain, Menu, globalShortcut, nativeImage } = require('electron');
const path = require('path');
const { spawn } = require('child_process');
const fs = require('fs');

let mainWindow;
let backendProcess;

// Minimal menu with DevTools
const template = [
    {
        label: 'DevTools',
        accelerator: 'CmdOrCtrl+Shift+I',
        click: () => mainWindow?.webContents.toggleDevTools()
    }
];
Menu.setApplicationMenu(Menu.buildFromTemplate(template));

let backendPort = 0;

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
        WEBTERM_PORT: ':0', // Request dynamic port
        WEBTERM_DB_PATH: dbPath,
        WEBTERM_ALLOWED_ORIGINS: '*', 
    };

    backendProcess = spawn(backendPath, [], { env });

    backendProcess.stdout.on('data', (data) => {
        const text = data.toString();
        console.log(`Backend: ${text}`);
        if (text.includes('BACKEND_PORT:')) {
            const port = parseInt(text.split('BACKEND_PORT:')[1].trim());
            if (!isNaN(port)) {
                backendPort = port;
                if (mainWindow) {
                    mainWindow.webContents.send('backend-ready', port);
                }
            }
        }
    });

    backendProcess.stderr.on('data', (data) => {
        console.error(`Backend Error: ${data}`);
    });

    backendProcess.on('close', (code) => {
        console.log(`Backend process exited with code ${code}`);
    });
}

// IPC Handlers
ipcMain.handle('get-backend-port', () => {
    return backendPort;
});

function createWindow() {
    const isWin = process.platform === 'win32';
    const isMac = process.platform === 'darwin';

    mainWindow = new BrowserWindow({
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        frame: false,
        // Windows 11 handles rounding natively for frameless windows but ONLY if transparent is false.
        // Linux/macOS need transparent: true to let the web-content's CSS rounding show through.
        transparent: !isWin,
        backgroundColor: isWin ? undefined : '#00000000',
        ...( (isWin || isMac) ? { titleBarStyle: 'hidden' } : {} ),
        titleBarOverlay: isWin ? {
            color: '#00000000', 
            symbolColor: '#94a3b8',
            height: 48 
        } : false,
        autoHideMenuBar: false,
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        },
        title: "WebTerm Desktop",
        icon: nativeImage.createFromPath(path.join(__dirname, 'assets', 'icon.png')),
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
