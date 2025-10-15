const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');

let mainWindow;
let dragPlugin;

// Import the drag plugin
try {
  dragPlugin = require('../index.js');
  console.log('✅ Drag plugin loaded successfully');
} catch (error) {
  console.error('❌ Failed to load drag plugin:', error);
  dragPlugin = null;
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
    title: 'Advanced Electron Drag File Plugin Example',
    show: false // Don't show until ready
  });

  // Load the HTML
  mainWindow.loadFile('index.html');

  // Show window when ready to prevent visual flash
  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });

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
    console.log('🚀 Starting drag monitoring...');

    // Start monitoring
    await dragPlugin.startDragMonitor();
    console.log('✅ Drag monitoring started');

    // Check initial status
    const isMonitoring = await dragPlugin.isMonitoring();
    console.log(`📊 Monitoring status: ${isMonitoring}`);

    // Register for drag events
    const callbackId = await dragPlugin.onDragEvent((event) => {
      console.log('🎯 Drag event detected:', {
        files: event.files,
        timestamp: event.timestamp,
        source: event.source,
        platform: event.platform,
        dragState: event.dragState,
        x: event.x,
        y: event.y
      });

      // Send event to renderer process with enhanced data
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('drag-event', {
          ...event,
          timestamp: new Date().toISOString(),
          windowFocused: mainWindow.isFocused()
        });
      }
    });

    console.log(`📝 Registered drag callback with ID: ${callbackId}`);

    // Test with simulated events periodically
    setInterval(() => {
      if (mainWindow && !mainWindow.isDestroyed()) {
        simulateRandomDragEvent();
      }
    }, 10000); // Every 10 seconds

    // Clean up on app exit
    app.on('before-quit', async () => {
      console.log('🛑 Stopping drag monitoring...');
      await dragPlugin.removeDragEventListener(callbackId);
      await dragPlugin.stopDragMonitor();
      console.log('✅ Drag monitoring stopped');
    });

  } catch (error) {
    console.error('❌ Failed to initialize drag monitoring:', error);

    // Show error dialog
    dialog.showErrorBox('Drag Plugin Error',
      `Failed to initialize drag monitoring: ${error.message}`);
  }
}

async function simulateRandomDragEvent() {
  if (!dragPlugin) return;

  const sampleFiles = [
    '/Users/test/Documents/sample.txt',
    '/Users/test/Pictures/image.jpg',
    '/Users/test/Videos/video.mp4',
    '/Users/test/Downloads/archive.zip'
  ];

  // Randomly select 1-3 files
  const numFiles = Math.floor(Math.random() * 3) + 1;
  const selectedFiles = sampleFiles
    .sort(() => Math.random() - 0.5)
    .slice(0, numFiles);

  try {
    await dragPlugin.simulateDragEvent(selectedFiles);
    console.log('📤 Sent simulated drag event:', selectedFiles);
  } catch (error) {
    console.error('❌ Failed to simulate drag event:', error);
  }
}

// Enhanced IPC handlers
ipcMain.handle('drag-monitor:start', async () => {
  if (!dragPlugin) {
    throw new Error('Drag plugin not available');
  }
  const result = await dragPlugin.startDragMonitor();
  return { success: true, result };
});

ipcMain.handle('drag-monitor:stop', async () => {
  if (!dragPlugin) {
    throw new Error('Drag plugin not available');
  }
  const result = await dragPlugin.stopDragMonitor();
  return { success: true, result };
});

ipcMain.handle('drag-monitor:status', async () => {
  if (!dragPlugin) {
    return { available: false, monitoring: false };
  }
  const isMonitoring = await dragPlugin.isMonitoring();
  return {
    available: true,
    monitoring: isMonitoring,
    platform: process.platform
  };
});

ipcMain.handle('drag-monitor:simulate', async (event, files) => {
  if (!dragPlugin) {
    throw new Error('Drag plugin not available');
  }

  if (!Array.isArray(files) || files.length === 0) {
    throw new Error('Files array must be a non-empty array');
  }

  try {
    await dragPlugin.simulateDragEvent(files);
    return { success: true, files };
  } catch (error) {
    throw new Error(`Failed to simulate drag event: ${error.message}`);
  }
});

// File system operations
ipcMain.handle('fs:readFile', async (event, filePath) => {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    return { success: true, content, path: filePath };
  } catch (error) {
    throw new Error(`Failed to read file: ${error.message}`);
  }
});

ipcMain.handle('fs:fileExists', async (event, filePath) => {
  try {
    const exists = fs.existsSync(filePath);
    return { exists, path: filePath };
  } catch (error) {
    return { exists: false, path: filePath, error: error.message };
  }
});

// App lifecycle
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

// Handle certificate errors for development
app.on('certificate-error', (event, webContents, url, error, certificate, callback) => {
  // On development, ignore certificate errors
  if (process.env.NODE_ENV === 'development') {
    event.preventDefault();
    callback(true);
  } else {
    callback(false);
  }
});

console.log('🚀 Advanced Electron Drag File Plugin Example Started');
console.log(`Platform: ${process.platform}`);
console.log(`Node.js: ${process.version}`);
console.log(`Electron: ${process.versions.electron}`);