const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');

let mainWindow;
let dragPlugin;

// Import the drag plugin (this will work after the native addon is built)
try {
  dragPlugin = require('electron-dragfile-plugin');
  console.log('Drag plugin loaded successfully');
} catch (error) {
  console.error('Failed to load drag plugin:', error);
  dragPlugin = null;
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1000,
    height: 700,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js')
    },
    title: 'Electron Drag File Plugin Example'
  });

  mainWindow.loadFile('index.html');

  if (process.argv.includes('--dev')) {
    mainWindow.webContents.openDevTools();
  }

  // Initialize drag monitoring if plugin is available
  if (dragPlugin) {
    initializeDragMonitoring();
  }
}

async function initializeDragMonitoring() {
  try {
    console.log('Starting drag monitoring...');
    await dragPlugin.startDragMonitor();
    console.log('Drag monitoring started');

    // Register for drag events
    const callbackId = await dragPlugin.onDragEvent((event) => {
      console.log('Drag event detected:', event);

      // Send event to renderer process
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('drag-event', event);
      }
    });

    console.log(`Registered drag callback with ID: ${callbackId}`);

    // Clean up on app exit
    app.on('before-quit', async () => {
      console.log('Stopping drag monitoring...');
      await dragPlugin.removeDragEventListener(callbackId);
      await dragPlugin.stopDragMonitor();
      console.log('Drag monitoring stopped');
    });

  } catch (error) {
    console.error('Failed to initialize drag monitoring:', error);
  }
}

// IPC handlers
ipcMain.handle('drag-monitor:start', async () => {
  if (!dragPlugin) {
    throw new Error('Drag plugin not available');
  }
  return await dragPlugin.startDragMonitor();
});

ipcMain.handle('drag-monitor:stop', async () => {
  if (!dragPlugin) {
    throw new Error('Drag plugin not available');
  }
  return await dragPlugin.stopDragMonitor();
});

ipcMain.handle('drag-monitor:status', async () => {
  if (!dragPlugin) {
    return false;
  }
  return await dragPlugin.isMonitoring();
});

ipcMain.handle('drag-monitor:simulate', async (event, files) => {
  if (!dragPlugin) {
    throw new Error('Drag plugin not available');
  }
  return await dragPlugin.simulateDragEvent(files);
});

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});