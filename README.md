# electron-dragfile-plugin

A high-performance native Node.js addon built with Rust and napi-rs that detects system-wide file drag events. Perfect for Electron applications that need to monitor file dragging across the entire system.

## ✨ Features

- 🚀 **High Performance**: Built with Rust for maximum performance and low overhead
- 🌍 **Cross-Platform**: Supports Windows and macOS (Linux planned)
- 📡 **System-Wide Detection**: Monitors drag events across the entire system, not just your app
- 🔧 **Easy to Use**: Simple JavaScript API with TypeScript support
- 📦 **NPM Ready**: Published to npm for easy installation
- 🎯 **Electron Ready**: Specifically designed for Electron applications

## 📦 Installation

```bash
npm install electron-dragfile-plugin
```

## 🚀 Quick Start

```javascript
const { startDragMonitor, onDragEvent } = require('electron-dragfile-plugin');

// Start monitoring drag events
await startDragMonitor();

// Listen for drag events
onDragEvent((event) => {
  console.log('Files dragged:', event.files);
  console.log('Timestamp:', event.timestamp);
  console.log('Source:', event.source);
});
```

## 📖 API Reference

### Functions

#### `startDragMonitor(): Promise<void>`
Start monitoring drag events globally.

#### `stopDragMonitor(): Promise<void>`
Stop monitoring drag events.

#### `onDragEvent(callback: Function): Promise<number>`
Register a callback for drag events. Returns a callback ID.

#### `removeDragEventListener(callbackId: number): Promise<boolean>`
Remove a drag event callback using the returned ID.

#### `isMonitoring(): Promise<boolean>`
Check if drag monitoring is currently active.

#### `simulateDragEvent(files: string[]): Promise<void>`
Simulate a drag event for testing purposes.

### DragEvent Interface

```typescript
interface DragEvent {
  files: string[];        // Array of file paths being dragged
  timestamp: number;      // Unix timestamp of the event
  source?: string;        // Optional source window information
}
```

### DragMonitor Class

For easier event management:

```javascript
const { DragMonitor } = require('electron-dragfile-plugin');

const monitor = new DragMonitor();

await monitor.start((event) => {
  console.log('Drag detected:', event);
});

// Later...
await monitor.stop();
```

## 🎯 Electron Integration

Here's how to integrate with your Electron app:

### Main Process (main.js)

```javascript
const { app, BrowserWindow, ipcMain } = require('electron');
const { startDragMonitor, onDragEvent } = require('electron-dragfile-plugin');

function createWindow() {
  const mainWindow = new BrowserWindow({
    // ... your window config
  });

  // Initialize drag monitoring
  await startDragMonitor();

  const callbackId = await onDragEvent((event) => {
    // Send drag events to renderer
    mainWindow.webContents.send('drag-event', event);
  });
}

app.whenReady().then(createWindow);
```

### Preload Script (preload.js)

```javascript
const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('dragAPI', {
  onDragEvent: (callback) => {
    ipcRenderer.on('drag-event', (event, data) => callback(data));
  }
});
```

### Renderer Process

```javascript
window.dragAPI.onDragEvent((event) => {
  console.log('Files were dragged:', event.files);
  // Update UI or handle drag events
});
```

## 🧪 Testing

```bash
# Install dependencies
npm install

# Run tests
npm test

# Run example Electron app
npm run example
```

## 🏗️ Development

### Prerequisites

- Node.js 14+
- Rust 1.70+

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/electron-dragfile-plugin.git
cd electron-dragfile-plugin

# Install dependencies
npm install

# Build the native addon
npm run build

# Run tests
npm test
```

### Project Structure

```
electron-dragfile-plugin/
├── src/
│   └── lib.rs              # Rust native code
├── index.js                # Node.js entry point
├── index.d.ts              # TypeScript definitions
├── Cargo.toml              # Rust project config
├── package.json            # NPM package config
├── example/                # Electron example app
├── test/                   # Test files
└── scripts/                # Build scripts
```

## 🔧 Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows 10+ | ✅ Supported | Full system-wide drag detection |
| macOS 10.14+ | ✅ Supported | System-wide drag monitoring |
| Linux | 🚧 Planned | Support planned for future releases |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [napi-rs](https://napi.rs/) for high-performance native addons
- Platform-specific APIs for system integration
- The Electron community for inspiration and feedback

## 📞 Support

If you encounter any issues or have questions:

- [GitHub Issues](https://github.com/yourusername/electron-dragfile-plugin/issues)
- [Discussions](https://github.com/yourusername/electron-dragfile-plugin/discussions)

---

Made with ❤️ for the Electron community
