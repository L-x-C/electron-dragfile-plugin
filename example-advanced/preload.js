const { contextBridge, ipcRenderer } = require('electron');

// Expose a safe API to the renderer process
contextBridge.exposeInMainWorld('electronDragAPI', {
  // Drag monitoring functions
  startDragMonitor: () => ipcRenderer.invoke('drag-monitor:start'),
  stopDragMonitor: () => ipcRenderer.invoke('drag-monitor:stop'),
  getMonitoringStatus: () => ipcRenderer.invoke('drag-monitor:status'),
  simulateDragEvent: (files) => ipcRenderer.invoke('drag-monitor:simulate', files),

  // Event listeners
  onDragEvent: (callback) => {
    const subscription = ipcRenderer.on('drag-event', (event, data) => {
      callback(data);
    });

    // Return unsubscribe function
    return () => {
      ipcRenderer.removeListener('drag-event', subscription);
    };
  },

  removeDragEventListener: () => {
    ipcRenderer.removeAllListeners('drag-event');
  },

  // File system helpers
  readFile: (filePath) => ipcRenderer.invoke('fs:readFile', filePath),
  fileExists: (filePath) => ipcRenderer.invoke('fs:fileExists', filePath),

  // System information
  getPlatform: () => process.platform,
  getVersions: () => ({
    node: process.versions.node,
    chrome: process.versions.chrome,
    electron: process.versions.electron
  }),

  // Utility functions
  formatFileSize: (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  },

  getFileExtension: (filename) => {
    return filename.slice((filename.lastIndexOf('.') - 1 >>> 0) + 2);
  },

  getFileIcon: (filename) => {
    const ext = filename.slice((filename.lastIndexOf('.') - 1 >>> 0) + 2).toLowerCase();

    const iconMap = {
      // Images
      jpg: '🖼️', jpeg: '🖼️', png: '🖼️', gif: '🖼️', svg: '🎨', ico: '🖼️',
      // Documents
      pdf: '📄', doc: '📝', docx: '📝', txt: '📄', rtf: '📄',
      // Spreadsheets
      xls: '📊', xlsx: '📊', csv: '📊',
      // Presentations
      ppt: '📽', pptx: '📽',
      // Archives
      zip: '🗜️', rar: '🗜️', tar: '🗜️', gz: '🗜️', 7z: '🗜️',
      // Code
      js: '📜', ts: '📜', html: '🌐', css: '🎨', json: '📋', xml: '📋',
      // Audio
      mp3: '🎵', wav: '🎵', flac: '🎵', aac: '🎵', ogg: '🎵',
      // Video
      mp4: '🎬', avi: '🎬', mkv: '🎬', mov: '🎬', wmv: '🎬',
      // Default
      default: '📄'
    };

    return iconMap[ext] || iconMap.default;
  }
});

// Expose node API helpers for debugging
contextBridge.exposeInMainWorld('nodeAPI', {
  // Safe process information
  getPlatform: () => process.platform,
  getArch: () => process.arch,
  getNodeVersion: () => process.versions.node,
  getElectronVersion: () => process.versions.electron,

  // Environment detection
  isDevelopment: () => process.env.NODE_ENV === 'development',
  isProduction: () => process.env.NODE_ENV === 'production',

  // Safe error handling
  handleError: (error, context) => {
    console.error(`[${context}] Error:`, error);
    return {
      message: error.message,
      stack: error.stack,
      context
    };
  }
});

// Development helpers
if (process.env.NODE_ENV === 'development') {
  contextBridge.exposeInMainWorld('devAPI', {
    log: (...args) => console.log('[Renderer]', ...args),
    error: (...args) => console.error('[Renderer]', ...args),
    warn: (...args) => console.warn('[Renderer]', ...args),

    // Performance monitoring
    mark: (name) => performance.mark(name),
    measure: (name, startMark, endMark) => performance.measure(name, startMark, endMark),

    // Memory usage
    getMemoryUsage: () => process.getMemoryUsage(),

    // Debug info
    getDebugInfo: () => ({
      userAgent: navigator.userAgent,
      language: navigator.language,
      cookieEnabled: navigator.cookieEnabled,
      onLine: navigator.onLine,
      platform: navigator.platform
    })
  });
}

console.log('🔗 Preload script loaded successfully');