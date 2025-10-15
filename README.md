# electron-dragfile-plugin

A high-performance native Node.js addon built with Rust and napi-rs that monitors system-wide mouse events. Perfect for applications that need to track mouse movements and clicks across the entire system.

## ✨ Features

- 🚀 **High Performance**: Built with Rust for maximum performance and low overhead
- 🌍 **Cross-Platform**: Supports macOS, Windows, and Linux
- 📡 **System-Wide Detection**: Monitors mouse events across the entire system, not just your app
- 🔧 **Easy to Use**: Simple JavaScript API with TypeScript support
- 📦 **NPM Ready**: Published to npm for easy installation
- 🎯 **Universal**: Works with any Node.js application, not just Electron

## 📦 Installation

```bash
npm install electron-dragfile-plugin
```

## 🚀 Quick Start

```javascript
const { startMouseMonitor, onMouseEvent } = require('electron-dragfile-plugin');

// Start monitoring mouse events
await startMouseMonitor();

// Listen for mouse events
onMouseEvent((err, event) => {
  if (err) {
    console.error('Error:', err);
    return;
  }

  const buttonName = event.button === 0 ? 'None' :
    event.button === 1 ? 'Left' :
    event.button === 2 ? 'Middle' :
    event.button === 3 ? 'Right' : `Button ${event.button}`;

  console.log(`🖱️ ${event.eventType.toUpperCase()} at (${event.x.toFixed(2)}, ${event.y.toFixed(2)}) - ${buttonName}`);
  console.log(`   Platform: ${event.platform}`);
  console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
});
```

## 📖 API Reference

### Functions

#### `startMouseMonitor(): Promise<void>`
Start monitoring mouse events globally.

#### `stopMouseMonitor(): Promise<void>`
Stop monitoring mouse events.

#### `onMouseEvent(callback: Function): Promise<number>`
Register a callback for mouse events. Returns a callback ID.

#### `removeMouseEventListener(callbackId: number): Promise<boolean>`
Remove a mouse event callback using the returned ID.

#### `isMonitoring(): Promise<boolean>`
Check if mouse monitoring is currently active.

### MouseEvent Interface

```typescript
interface MouseEvent {
  eventType: string;      // Event type: "mousedown", "mouseup", "mousemove", "wheel"
  x: number;             // Mouse X coordinate
  y: number;             // Mouse Y coordinate
  button: number;        // Mouse button: 0=no button, 1=left, 2=middle, 3=right
  timestamp: number;     // Unix timestamp of the event
  platform: string;     // Platform information: "macos", "windows", "linux"
}
```

## 🎯 Application Integration

Here's how to integrate with your Node.js application:

### Basic Usage

```javascript
const { startMouseMonitor, onMouseEvent, stopMouseMonitor } = require('electron-dragfile-plugin');

async function setupMouseTracking() {
  try {
    // Start monitoring
    await startMouseMonitor();
    console.log('✅ Mouse monitoring started');

    // Register callback
    const callbackId = await onMouseEvent((err, event) => {
      if (err) {
        console.error('Mouse event error:', err);
        return;
      }

      console.log(`🖱️ ${event.eventType} at (${event.x}, ${event.y})`);
      // Handle mouse events here
    });

    console.log('✅ Callback registered with ID:', callbackId);

    // Later, you can stop monitoring
    // await stopMouseMonitor();

  } catch (error) {
    console.error('❌ Failed to setup mouse monitoring:', error);
  }
}

setupMouseTracking();
```

### Error Handling

```javascript
const { startMouseMonitor, onMouseEvent } = require('electron-dragfile-plugin');

async function robustMouseTracking() {
  try {
    await startMouseMonitor();

    const callbackId = await onMouseEvent((err, event) => {
      if (err) {
        console.error('Callback error:', err);
        return;
      }

      if (!event) {
        console.warn('Received null event');
        return;
      }

      // Process event safely
      processMouseEvent(event);
    });

  } catch (setupError) {
    console.error('Setup failed:', setupError);
  }
}

function processMouseEvent(event) {
  try {
    // Your event processing logic here
    console.log(`Mouse event: ${event.eventType} at (${event.x}, ${event.y})`);
  } catch (processingError) {
    console.error('Event processing error:', processingError);
  }
}
```

## 🧪 Testing

```bash
# Install dependencies
npm install

# Build the native addon
npm run build

# Run the test script
node test.js
```

The test script will start mouse monitoring and log all mouse events to the console.

## 🔧 Platform Requirements

### macOS
- Requires macOS 10.14 or later
- May need to grant Accessibility permissions for global mouse monitoring
  - Go to System Preferences → Security & Privacy → Privacy → Accessibility
  - Add your terminal or Node.js application to the list

### Windows
- Requires Windows 10 or later
- No additional permissions required

### Linux
- Requires X11 display server
- No additional permissions required

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
├── test.js                 # Test script
└── README.md               # This file
```

## 🔧 Platform Support

| Platform | Status | Binary |
|----------|--------|--------|
| Windows 10+ (x64) | ✅ Supported | electron-dragfile-plugin.win32-x64-msvc.node |
| macOS 10.14+ (Intel) | ✅ Supported | electron-dragfile-plugin.darwin-x64.node |
| macOS 11+ (Apple Silicon) | ✅ Supported | electron-dragfile-plugin.darwin-arm64.node |
| Linux (x64) | ✅ Supported | electron-dragfile-plugin.linux-x64-gnu.node |

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
- Uses [rdev](https://github.com/narsil/rdev) for cross-platform mouse event monitoring
- Platform-specific APIs for system integration

## 📞 Support

If you encounter any issues or have questions:

- [GitHub Issues](https://github.com/yourusername/electron-dragfile-plugin/issues)
- [Discussions](https://github.com/yourusername/electron-dragfile-plugin/discussions)

---

Made with ❤️ for the Node.js community