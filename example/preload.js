const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronDragAPI', {
  // Drag monitoring functions
  startDragMonitor: () => ipcRenderer.invoke('drag-monitor:start'),
  stopDragMonitor: () => ipcRenderer.invoke('drag-monitor:stop'),
  getMonitoringStatus: () => ipcRenderer.invoke('drag-monitor:status'),
  simulateDragEvent: (files) => ipcRenderer.invoke('drag-monitor:simulate', files),

  // Event listeners
  onDragEvent: (callback) => {
    ipcRenderer.on('drag-event', (event, data) => callback(data));
  },

  removeDragEventListener: () => {
    ipcRenderer.removeAllListeners('drag-event');
  }
});