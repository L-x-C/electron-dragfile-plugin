# electron-dragfile-plugin

A high-performance native Node.js addon built with Rust and napi-rs that monitors system-wide mouse and keyboard events. Perfect for applications that need to track mouse movements, clicks, and keyboard inputs across the entire system.

## ✨ Features

- 🚀 **High Performance**: Built with Rust for maximum performance and low overhead
- 🌍 **Cross-Platform**: Supports macOS, Windows, and Linux
- 📡 **System-Wide Detection**: Monitors mouse and keyboard events across the entire system, not just your app
- ⌨️ **Full Keyboard Support**: Detects key presses, releases, and modifier keys
- 🖱️ **Complete Mouse Tracking**: Tracks mouse movements, clicks, and wheel events
- 🔧 **Easy to Use**: Simple JavaScript API with TypeScript support
- 📦 **NPM Ready**: Published to npm for easy installation
- 🎯 **Universal**: Works with any Node.js application, not just Electron

## 📦 Installation

```bash
npm install electron-dragfile-plugin
```

## 🚀 Quick Start

```javascript
const {
  startMouseMonitor,
  startKeyboardMonitor,
  onMouseEvent,
  onKeyboardEvent
} = require('electron-dragfile-plugin');

// Start monitoring mouse and keyboard events
await startMouseMonitor();
await startKeyboardMonitor();

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

// Listen for keyboard events
onKeyboardEvent((err, event) => {
  if (err) {
    console.error('Error:', err);
    return;
  }

  const modifiers = event.modifiers.length > 0 ? event.modifiers.join('+') + '+' : '';
  console.log(`⌨️ ${event.eventType.toUpperCase()}: ${modifiers}${event.keyName} (code: ${event.keyCode})`);
  console.log(`   Platform: ${event.platform}`);
  console.log(`   Time: ${new Date(event.timestamp * 1000).toLocaleTimeString()}`);
});
```

## 📖 API Reference

### Mouse Event Functions

#### `startMouseMonitor(): Promise<void>`
Start monitoring mouse events globally.

#### `stopMouseMonitor(): Promise<void>`
Stop monitoring mouse events.

#### `onMouseEvent(callback: Function): Promise<number>`
Register a callback for mouse events. Returns a callback ID.

#### `removeMouseEventListener(callbackId: number): Promise<boolean>`
Remove a mouse event callback using the returned ID.

### Keyboard Event Functions

#### `startKeyboardMonitor(): Promise<void>`
Start monitoring keyboard events globally.

#### `stopKeyboardMonitor(): Promise<void>`
Stop monitoring keyboard events.

#### `onKeyboardEvent(callback: Function): Promise<number>`
Register a callback for keyboard events. Returns a callback ID.

#### `removeKeyboardEventListener(callbackId: number): Promise<boolean>`
Remove a keyboard event callback using the returned ID.

### Status Functions

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

### KeyboardEvent Interface

```typescript
interface KeyboardEvent {
  eventType: string;      // Event type: "keydown", "keyup"
  keyCode: number;        // Keyboard scan code
  keyName: string;        // Human-readable key name: "A", "F1", "Shift", etc.
  modifiers: string[];    // Active modifier keys: ["shift"], ["control", "alt"]
  timestamp: number;      // Unix timestamp of the event
  platform: string;      // Platform information: "macos", "windows", "linux"
}
```

## 🎯 Application Integration

Here's how to integrate with your Node.js application:

### Basic Usage - Mouse and Keyboard

```javascript
const {
  startMouseMonitor,
  startKeyboardMonitor,
  onMouseEvent,
  onKeyboardEvent,
  stopMouseMonitor,
  stopKeyboardMonitor
} = require('electron-dragfile-plugin');

async function setupInputTracking() {
  try {
    // Start monitoring
    await startMouseMonitor();
    await startKeyboardMonitor();
    console.log('✅ Mouse and keyboard monitoring started');

    // Register mouse callback
    const mouseCallbackId = await onMouseEvent((err, event) => {
      if (err) {
        console.error('Mouse event error:', err);
        return;
      }

      console.log(`🖱️ ${event.eventType} at (${event.x}, ${event.y})`);
      // Handle mouse events here
    });

    // Register keyboard callback
    const keyboardCallbackId = await onKeyboardEvent((err, event) => {
      if (err) {
        console.error('Keyboard event error:', err);
        return;
      }

      const modifiers = event.modifiers.length > 0 ? event.modifiers.join('+') + '+' : '';
      console.log(`⌨️ ${modifiers}${event.keyName} (${event.eventType})`);
      // Handle keyboard events here
    });

    console.log('✅ Callbacks registered - Mouse ID:', mouseCallbackId, 'Keyboard ID:', keyboardCallbackId);

    // Later, you can stop monitoring
    // await stopMouseMonitor();
    // await stopKeyboardMonitor();

  } catch (error) {
    console.error('❌ Failed to setup input monitoring:', error);
  }
}

setupInputTracking();
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

The test script will start both mouse and keyboard monitoring and log all events to the console.

**Test Instructions:**
1. Run `node test.js`
2. Move your mouse around - you'll see mouse movement events
3. Click different mouse buttons - you'll see click events with coordinates
4. Press various keys on your keyboard - you'll see keydown and keyup events
5. Try modifier keys (Shift, Ctrl, Alt) with other keys
6. Press Ctrl+C to stop the test

**Example Output:**
```
🖱️ MOUSEMOVE at (245.67, 189.23) - None
   Platform: macos
   Time: 14:30:25
---
⌨️ KEYDOWN: A (code: 65)
   Platform: macos
   Time: 14:30:26
---
⌨️ KEYUP: A (code: 65)
   Platform: macos
   Time: 14:30:26
---
⌨️ KEYDOWN: shift+A (code: 65)
   Platform: macos
   Time: 14:30:30
---
```

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
- Uses [rdev](https://github.com/narsil/rdev) for cross-platform mouse and keyboard event monitoring
- Platform-specific APIs for system integration

## 📞 Support

If you encounter any issues or have questions:

- [GitHub Issues](https://github.com/yourusername/electron-dragfile-plugin/issues)
- [Discussions](https://github.com/yourusername/electron-dragfile-plugin/discussions)

---

Made with ❤️ for the Node.js community