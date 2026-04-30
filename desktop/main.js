const { app, BrowserWindow } = require('electron');
const path = require('path');
const { spawn } = require('child_process');
const fs = require('fs');

let mainWindow;
let backendProcess;

function startBackend() {
    const isDev = process.env.NODE_ENV === 'development';
    
    // Path to the backend binary
    // In dev, we assume it's built in be/
    // In production, it would be bundled with the app
    let backendPath = path.join(__dirname, '..', 'be', 'webterm-backend');
    if (process.platform === 'win32') {
        backendPath += '.exe';
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
    mainWindow = new BrowserWindow({
        width: 1200,
        height: 800,
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
            preload: path.join(__dirname, 'preload.js')
        },
        title: "WebTerm Desktop",
        backgroundColor: '#000000'
    });

    // In dev, we might want to load from Vite dev server
    // In prod, load the built files
    const isDev = process.env.NODE_ENV === 'development';
    
    if (isDev) {
        mainWindow.loadURL('http://localhost:5173');
        mainWindow.webContents.openDevTools();
    } else {
        mainWindow.loadFile(path.join(__dirname, '..', 'fe', 'dist', 'index.html'));
    }

    mainWindow.on('closed', function () {
        mainWindow = null;
    });
}

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
